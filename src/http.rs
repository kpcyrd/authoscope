use crate::errors::*;
use crate::structs::LuaMap;

use reqwest::Method;
use reqwest::header::{HeaderName, HeaderValue, COOKIE, SET_COOKIE, USER_AGENT};
use reqwest::redirect;
use crate::hlua::AnyLuaValue;
use crate::json::LuaJsonValue;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use crate::config::Config;
use crate::ctx::State;
use crate::utils;

#[derive(Debug)]
pub struct HttpSession {
    id: String,
    pub cookies: CookieJar,
}

impl HttpSession {
    pub fn new() -> (String, HttpSession) {
        let id = utils::random_string(16);
        (id.clone(), HttpSession {
            id,
            cookies: CookieJar::default(),
        })
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct RequestOptions {
    query: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
    basic_auth: Option<(String, String)>,
    user_agent: Option<String>,
    json: Option<serde_json::Value>,
    form: Option<serde_json::Value>,
    body: Option<String>,
}

impl RequestOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<RequestOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HttpRequest {
    // reference to the HttpSession
    session: String,
    cookies: CookieJar,
    method: String,
    url: String,
    query: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
    basic_auth: Option<(String, String)>,
    user_agent: Option<String>,
    body: Option<Body>,
}

impl HttpRequest {
    pub fn new(config: &Arc<Config>, session: &HttpSession, method: String, url: String, options: RequestOptions) -> HttpRequest {
        let cookies = session.cookies.clone();

        let user_agent = options.user_agent.or_else(|| config.runtime.user_agent.clone());

        let mut request = HttpRequest {
            session: session.id.clone(),
            cookies,
            method,
            url,
            query: options.query,
            headers: options.headers,
            basic_auth: options.basic_auth,
            user_agent,
            body: None,
        };

        if let Some(json) = options.json {
            request.body = Some(Body::Json(json));
        }

        if let Some(form) = options.form {
            request.body = Some(Body::Form(form));
        }

        if let Some(text) = options.body {
            request.body = Some(Body::Raw(text));
        }

        request
    }

    pub fn send(&self, state: &State) -> Result<LuaMap> {
        debug!("http send: {:?}", self);

        let client = reqwest::blocking::Client::builder()
            .redirect(redirect::Policy::none()) // TODO: this should be configurable
            .build().unwrap();
        let method = self.method.parse::<Method>()
                        .context("Invalid http method")?;
        let mut req = client.request(method, &self.url);

        if let Some(cookies) = self.cookies.assemble_cookie_header() {
            debug!("Adding cookies to request: {:?}", cookies);
            req = req.header(COOKIE, HeaderValue::from_str(&cookies)?);
        }

        if let Some(ref agent) = self.user_agent {
            req = req.header(USER_AGENT, agent.as_str());
        }

        if let Some(ref auth) = self.basic_auth {
            let &(ref user, ref password) = auth;
            req = req.basic_auth(user, Some(password));
        }

        if let Some(ref headers) = self.headers {
            for (k, v) in headers {
                let k = HeaderName::from_bytes(k.as_bytes())?;
                req = req.header(k, HeaderValue::from_str(v)?);
            }
        }

        if let Some(ref query) = self.query {
            req = req.query(query);
        }

        req = match self.body {
            Some(Body::Raw(ref x))  => { req.body(x.clone()) },
            Some(Body::Form(ref x)) => { req.form(x) },
            Some(Body::Json(ref x)) => { req.json(x) },
            None => req,
        };

        info!("http req: {:?}", req);
        let res = req.send()?;
        info!("http res: {:?}", res);

        let mut resp = LuaMap::new();
        let status = res.status();
        resp.insert_num("status", f64::from(status.as_u16()));

        {
            let cookies = res.headers().get_all(SET_COOKIE);
            HttpRequest::register_cookies_on_state(&self.session, state, &cookies)
                .context("Failed to process http response cookies")?;
        }

        let mut headers = LuaMap::new();
        for (name, value) in res.headers().iter() {
            headers.insert_str(name.as_str().to_lowercase(), value.to_str()?);
        }
        resp.insert("headers", headers);

        if let Ok(text) = res.text() {
            resp.insert_str("text", text);
        }

        Ok(resp)
    }

    fn register_cookies_on_state(session: &str, state: &State, cookies: &reqwest::header::GetAll<HeaderValue>) -> Result<()> {
        let mut jar = Vec::new();

        for cookie in cookies.iter() {
            let cookie = cookie.to_str()?;

            let mut key = String::new();
            let mut value = String::new();
            let mut in_key = true;

            for c in cookie.chars() {
                match c {
                    '=' if in_key => in_key = false,
                    ';' => break,
                    c if in_key => key.push(c),
                    c => value.push(c),
                }
            }

            jar.push((key, value));
        }

        state.register_in_jar(session, jar);

        Ok(())
    }
}

impl HttpRequest {
    pub fn try_from(x: AnyLuaValue) -> Result<HttpRequest> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

impl From<HttpRequest> for AnyLuaValue {
    fn from(x: HttpRequest) -> AnyLuaValue {
        let v = serde_json::to_value(&x).unwrap();
        LuaJsonValue::from(v).into()
    }
}

// see https://github.com/seanmonstar/reqwest/issues/14 for proper cookie jars
// maybe change this to reqwest::header::Cookie
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CookieJar(HashMap<String, String>);

impl CookieJar {
    pub fn register_in_jar(&mut self, cookies: Vec<(String, String)>) {
        for (key, value) in cookies {
            self.0.insert(key, value);
        }
    }

    pub fn assemble_cookie_header(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let mut cookies: Vec<String> = Vec::new();
        for (key, value) in self.iter() {
            let value = if value.contains(' ') || value.contains(';') {
                self.escape_cookie_value(value)
            } else {
                value.to_owned()
            };

            let cookie = format!("{}={}", key, value);
            debug!("Adding cookie: {:?}", cookie);
            cookies.push(cookie);
        }

        Some(cookies.join("; "))
    }

    fn escape_cookie_value(&self, value: &str) -> String {
        value.chars()
            .fold(String::new(), |mut s, c| {
                match c {
                    ';' => s.push_str("\\073"),
                    c => s.push(c),
                }
                s
            })
    }
}

impl Deref for CookieJar {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Body {
    Raw(String), // TODO: maybe Vec<u8>
    Form(serde_json::Value),
    Json(serde_json::Value),
}
