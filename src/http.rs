use errors::Result;
use structs::LuaMap;

use reqwest;
use reqwest::header::Cookie;
use reqwest::header::UserAgent;
use hlua::AnyLuaValue;
use serde_json;
use json::LuaJsonValue;
use std::collections::HashMap;
use std::ops::Deref;
use rand::{Rng, thread_rng};
use ctx::State;


#[derive(Debug)]
pub struct HttpSession {
    id: String,
    pub cookies: CookieJar,
}

impl HttpSession {
    pub fn new() -> (String, HttpSession) {
        let id: String = thread_rng().gen_ascii_chars().take(16).collect();
        (id.clone(), HttpSession {
            id: id,
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

#[derive(Debug, Default)]
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
    pub fn new(session: &HttpSession, method: String, url: String, options: RequestOptions) -> (String, HttpRequest) {
        let id = thread_rng().gen_ascii_chars().take(16).collect();
        let cookies = session.cookies.clone();

        let mut request = HttpRequest {
            session: session.id.clone(),
            cookies,
            method,
            url,
            query: options.query,
            headers: options.headers,
            basic_auth: options.basic_auth,
            user_agent: options.user_agent,
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

        (id, request)
    }

    pub fn send(&self, state: &State) -> Result<LuaMap> {
        // println!("sending! {:?}", self);
        let client = reqwest::Client::builder()
            .redirect(reqwest::RedirectPolicy::none()) // TODO: this should be configurable
            .build().unwrap();
        let method = self.method.parse()?;
        let mut req = client.request(method, &self.url);

        let mut cookie = Cookie::new();
        for (key, value) in self.cookies.iter() {
            cookie.append(key.clone(), value.clone());
        }
        req.header(cookie);

        if let Some(ref agent) = self.user_agent {
            req.header(UserAgent::new(agent.clone()));
        }

        if let Some(ref query) = self.query {
            req.query(query);
        }

        match self.body {
            Some(Body::Raw(ref x))  => { req.body(x.clone()); },
            Some(Body::Form(ref x)) => { req.form(x); },
            Some(Body::Json(ref x)) => { req.json(x); },
            None => (),
        };

        // println!("req: {:?}", req);
        let mut res = req.send()?;
        // println!("res: {:?}", res);

        let mut resp = LuaMap::new();
        let status = res.status();
        resp.insert_num("status", status.as_u16() as f64);

        if let Some(cookies) = res.headers().get_raw("set-cookie") {
            HttpRequest::register_cookies_on_state(&self.session, state, cookies);
        }

        let mut headers = LuaMap::new();
        for header in res.headers().iter() {
            headers.insert_str(header.name().to_lowercase(), header.value_string());
        }
        resp.insert("headers", headers);

        if let Ok(text) = res.text() {
            resp.insert_str("text", text);
        }

        Ok(resp)
    }

    fn register_cookies_on_state(session: &String, state: &State, cookies: &reqwest::header::Raw) {
        let mut jar = Vec::new();

        for cookie in cookies {
            let mut key = String::new();
            let mut value = String::new();
            let mut in_key = true;

            for c in cookie.iter() {
                match *c as char {
                    '=' if in_key => in_key = false,
                    ';' => break,
                    c if in_key => key.push(c),
                    c => value.push(c),
                }
            }

            jar.push((key, value));
        }

        state.register_in_jar(session, jar);
    }
}

// see https://github.com/seanmonstar/reqwest/issues/14 for proper cookie jars
// maybe change this to reqwest::header::Cookie
#[derive(Debug, Default, Clone)]
pub struct CookieJar(HashMap<String, String>);

impl CookieJar {
    pub fn register_in_jar(&mut self, cookies: Vec<(String, String)>) {
        for (key, value) in cookies {
            self.0.insert(key, value);
        }
    }
}

impl Deref for CookieJar {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum Body {
    Raw(String), // TODO: maybe Vec<u8>
    Form(serde_json::Value),
    Json(serde_json::Value),
}
