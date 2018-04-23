use hlua;
use hlua::{AnyLuaValue, AnyHashableLuaValue, AnyLuaString};
use hlua::AnyLuaValue::LuaString;
use structs::LuaMap;
use errors::{Result, ResultExt};
use json;
use db;

use md5;
use sha1;
use sha2;
use sha3::{self, Digest};
use digest::{Input, BlockInput, FixedOutput};
use digest::generic_array::ArrayLength;
use hmac::{Hmac, Mac};
use base64;
use bcrypt;

use reqwest;
use ldap3;
use mysql;
use rand;
use rand::Rng;

use std::thread;
use std::time::Duration;
use std::process::Command;
use std::collections::HashMap;
use ctx::State;
use http::HttpRequest;
use http::RequestOptions;
use html;


fn byte_array(bytes: AnyLuaValue) -> Result<Vec<u8>> {
    match bytes {
        AnyLuaValue::LuaAnyString(bytes) => Ok(bytes.0),
        AnyLuaValue::LuaString(bytes) => Ok(bytes.into_bytes()),
        AnyLuaValue::LuaArray(bytes) => {
            Ok(bytes.into_iter()
                .map(|num| match num.1 {
                    AnyLuaValue::LuaNumber(num) if num <= 255.0 && num >= 0.0 && (num % 1.0 == 0.0) =>
                            Ok(num as u8),
                    AnyLuaValue::LuaNumber(num) =>
                            Err(format!("number is out of range: {:?}", num).into()),
                    _ => Err(format!("unexpected type: {:?}", num).into()),
                })
                .collect::<Result<_>>()?)
        },
        _ => Err(format!("invalid type: {:?}", bytes).into()),
    }
}

pub fn lua_bytes(bytes: &[u8]) -> AnyLuaValue {
    let bytes = AnyLuaString(bytes.to_vec());
    AnyLuaValue::LuaAnyString(bytes)
}


pub fn base64_decode(lua: &mut hlua::Lua, state: State) {
    lua.set("base64_decode", hlua::function1(move |bytes: String| -> Result<AnyLuaValue> {
        let bytes = match base64::decode(&bytes) {
            Ok(bytes) => bytes,
            Err(err) => return Err(state.set_error(err.into())),
        };

        Ok(lua_bytes(&bytes))
    }))
}

pub fn base64_encode(lua: &mut hlua::Lua, state: State) {
    lua.set("base64_encode", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        let bytes = match byte_array(bytes) {
            Ok(bytes) => bytes,
            Err(err) => return Err(state.set_error(err)),
        };

        Ok(base64::encode(&bytes))
    }))
}

pub fn bcrypt(lua: &mut hlua::Lua, state: State) {
    lua.set("bcrypt", hlua::function2(move |password: String, cost: u32| -> Result<String> {
        let result = match bcrypt::hash(&password, cost) {
            Ok(result) => result,
            Err(err) => return Err(state.set_error(err.into())),
        };

        Ok(result)
    }))
}

pub fn bcrypt_verify(lua: &mut hlua::Lua, state: State) {
    lua.set("bcrypt_verify", hlua::function2(move |password: String, hashed: String| -> Result<bool> {
        let result = match bcrypt::verify(&password, &hashed) {
            Ok(result) => result,
            Err(err) => return Err(state.set_error(err.into())),
        };

        Ok(result)
    }))
}

pub fn execve(lua: &mut hlua::Lua, state: State) {
    lua.set("execve", hlua::function2(move |prog: String, args: Vec<AnyLuaValue>| -> Result<i32> {
        let args: Vec<_> = args.into_iter()
                    .flat_map(|x| match x {
                        LuaString(x) => Some(x),
                        _ => None, // TODO: error
                    })
                    .collect();

        let status = match Command::new(prog)
                        .args(&args)
                        .status()
                        .chain_err(|| "failed to spawn program") {
            Ok(status) => status,
            Err(err) => return Err(state.set_error(err)),
        };

        let code = match status.code() {
            Some(code) => code,
            None => return Err(state.set_error("process didn't return exit code".into())),
        };

        Ok(code)
    }))
}

pub fn hex(lua: &mut hlua::Lua, state: State) {
    lua.set("hex", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        let bytes = match byte_array(bytes) {
            Ok(bytes) => bytes,
            Err(err) => return Err(state.set_error(err)),
        };

        let mut out = String::new();

        for b in bytes {
            out += &format!("{:02x}", b);
        }

        Ok(out)
    }))
}

fn hmac<D>(secret: AnyLuaValue, msg: AnyLuaValue) -> Result<AnyLuaValue>
    where
        D: Input + BlockInput + FixedOutput + Default + Clone,
        D::BlockSize: ArrayLength<u8>,
{
    let secret = byte_array(secret)?;
    let msg = byte_array(msg)?;

    let mut mac = match Hmac::<D>::new_varkey(&secret) {
        Ok(mac) => mac,
        Err(_) => return Err("invalid key length".into()),
    };
    mac.input(&msg);
    let result = mac.result();
    Ok(lua_bytes(&result.code()))
}

pub fn hmac_md5(lua: &mut hlua::Lua, state: State) {
    lua.set("hmac_md5", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<md5::Md5>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha1(lua: &mut hlua::Lua, state: State) {
    lua.set("hmac_sha1", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha1::Sha1>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha2_256(lua: &mut hlua::Lua, state: State) {
    lua.set("hmac_sha2_256", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha2::Sha256>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha2_512(lua: &mut hlua::Lua, state: State) {
    lua.set("hmac_sha2_512", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha2::Sha512>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha3_256(lua: &mut hlua::Lua, state: State) {
    lua.set("hmac_sha3_256", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha3::Sha3_256>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn hmac_sha3_512(lua: &mut hlua::Lua, state: State) {
    lua.set("hmac_sha3_512", hlua::function2(move |secret: AnyLuaValue, msg: AnyLuaValue| -> Result<AnyLuaValue> {
        hmac::<sha3::Sha3_512>(secret, msg)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn html_select(lua: &mut hlua::Lua, state: State) {
    lua.set("html_select", hlua::function2(move |html: String, selector: String| -> Result<AnyLuaValue> {
        html::html_select(&html, &selector)
            .map(|x| x.into())
            .map_err(|err| state.set_error(err))
    }))
}

pub fn html_select_list(lua: &mut hlua::Lua, state: State) {
    lua.set("html_select_list", hlua::function2(move |html: String, selector: String| -> Result<Vec<AnyLuaValue>> {
        html::html_select_list(&html, &selector)
            .map(|x| x.into_iter().map(|x| x.into()).collect())
            .map_err(|err| state.set_error(err))
    }))
}

pub fn http_basic_auth(lua: &mut hlua::Lua, state: State) {
    lua.set("http_basic_auth", hlua::function3(move |url: String, user: String, password: String| -> Result<bool> {
        let client = reqwest::Client::new();

        let response = match client.get(&url)
                             .basic_auth(user, Some(password))
                             .send()
                             .chain_err(|| "http request failed") {
            Ok(response) => response,
            Err(err) => return Err(state.set_error(err)),
        };

        // println!("{:?}", response);
        // println!("{:?}", response.headers().get_raw("www-authenticate"));
        // println!("{:?}", response.status());

        let authorized = response.headers().get_raw("www-authenticate").is_none() &&
            response.status() != reqwest::StatusCode::Unauthorized;

        Ok(authorized)
    }))
}

pub fn http_mksession(lua: &mut hlua::Lua, state: State) {
    lua.set("http_mksession", hlua::function0(move || -> String {
        state.http_mksession()
    }))
}

pub fn http_request(lua: &mut hlua::Lua, state: State) {
    lua.set("http_request", hlua::function4(move |session: String, method: String, url: String, options: AnyLuaValue| -> Result<AnyLuaValue> {
        let options = match RequestOptions::try_from(options)
                                .chain_err(|| "invalid request options") {
            Ok(options) => options,
            Err(err) => return Err(state.set_error(err)),
        };

        let req = state.http_request(&session, method, url, options);
        Ok(req.into())
    }))
}

pub fn http_send(lua: &mut hlua::Lua, state: State) {
    lua.set("http_send", hlua::function1(move |request: AnyLuaValue| -> Result<HashMap<AnyHashableLuaValue, AnyLuaValue>> {
        let req = match HttpRequest::try_from(request)
                                .chain_err(|| "invalid http request object") {
            Ok(req) => req,
            Err(err) => return Err(state.set_error(err)),
        };

        req.send(&state)
            .map(|resp| resp.into())
            .map_err(|err| state.set_error(err))
    }))
}

pub fn json_decode(lua: &mut hlua::Lua, state: State) {
    lua.set("json_decode", hlua::function1(move |x: String| -> Result<AnyLuaValue> {
        json::decode(&x)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn json_encode(lua: &mut hlua::Lua, state: State) {
    lua.set("json_encode", hlua::function1(move |x: AnyLuaValue| -> Result<String> {
        json::encode(x)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn last_err(lua: &mut hlua::Lua, state: State) {
    lua.set("last_err", hlua::function0(move || -> AnyLuaValue {
        match state.last_error() {
            Some(err) => AnyLuaValue::LuaString(err),
            None => AnyLuaValue::LuaNil,
        }
    }))
}

pub fn ldap_bind(lua: &mut hlua::Lua, state: State) {
    lua.set("ldap_bind", hlua::function3(move |url: String, dn: String, password: String| -> Result<bool> {
        let sock = match ldap3::LdapConn::new(&url)
                        .chain_err(|| "ldap connection failed") {
            Ok(sock) => sock,
            Err(err) => return Err(state.set_error(err)),
        };

        let result = match sock.simple_bind(&dn, &password)
                            .chain_err(|| "fatal error during simple_bind") {
            Ok(result) => result,
            Err(err) => return Err(state.set_error(err)),
        };

        // println!("{:?}", result);

        Ok(result.success().is_ok())
    }))
}

pub fn ldap_escape(lua: &mut hlua::Lua, _: State) {
    lua.set("ldap_escape", hlua::function1(move |s: String| -> String {
        ldap3::dn_escape(s).to_string()
    }))
}

pub fn ldap_search_bind(lua: &mut hlua::Lua, state: State) {
    lua.set("ldap_search_bind", hlua::function6(move |url: String, search_user: String, search_pw: String, base_dn: String, user: String, password: String| -> Result<bool> {

        let sock = match ldap3::LdapConn::new(&url)
                        .chain_err(|| "ldap connection failed") {
            Ok(sock) => sock,
            Err(err) => return Err(state.set_error(err)),
        };

        let result = match sock.simple_bind(&search_user, &search_pw)
                            .chain_err(|| "fatal error during simple_bind with search user") {
            Ok(result) => result,
            Err(err) => return Err(state.set_error(err)),
        };

        if result.success().is_err() {
            return Err("login with search user failed".into());
        }

        let search = format!("uid={}", ldap3::dn_escape(user));
        let result = match sock.search(&base_dn, ldap3::Scope::Subtree, &search, vec!["*"])
                            .chain_err(|| "fatal error during ldap search") {
            Ok(result) => result,
            Err(err) => return Err(state.set_error(err)),
        };

        let entries = match result.success()
                            .chain_err(|| "ldap search failed") {
            Ok(result) => result.0,
            Err(err) => return Err(state.set_error(err)),
        };

        // take the first result
        if let Some(entry) = entries.into_iter().next() {
            let entry = ldap3::SearchEntry::construct(entry);

            // we got the DN, try to login
            let result = match sock.simple_bind(&entry.dn, &password)
                                .chain_err(|| "fatal error during simple_bind") {
                Ok(result) => result,
                Err(err) => return Err(state.set_error(err)),
            };

            // println!("{:?}", result);

            Ok(result.success().is_ok())
        } else {
            return Ok(false);
        }
    }))
}

pub fn md5(lua: &mut hlua::Lua, state: State) {
    lua.set("md5", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map(|bytes| lua_bytes(&md5::Md5::digest(&bytes)))
            .map_err(|err| state.set_error(err))
    }))
}

pub fn mysql_connect(lua: &mut hlua::Lua, state: State) {
    lua.set("mysql_connect", hlua::function4(move |host: String, port: u16, user: String, password: String| -> Result<String> {
        let mut builder = mysql::OptsBuilder::new();
        builder.ip_or_hostname(Some(host))
               .tcp_port(port)
               .prefer_socket(false)
               .user(Some(user))
               .pass(Some(password));

        let sock = match mysql::Conn::new(builder) {
            Ok(sock) => sock,
            // TODO: setting an error here means we can't bruteforce mysql anymore
            Err(err) => return Err(state.set_error(err.into())),
        };

        let id = state.mysql_register(sock);
        Ok(id)
    }))
}

pub fn mysql_query(lua: &mut hlua::Lua, state: State) {
    lua.set("mysql_query", hlua::function3(move |session: String, query: String, params: HashMap<AnyHashableLuaValue, AnyLuaValue>| -> Result<Vec<AnyLuaValue>> {
        let params = LuaMap::from(params);

        let sock = state.mysql_session(&session);
        let mut sock = sock.lock().unwrap();
        let rows = sock.prep_exec(query, params)?; // TODO: handle error

        let mut result = Vec::new();
        let column_names = rows.column_indexes();

        for row in rows {
            let row = row?; // TODO: handle error

            let mut map = LuaMap::new();
            for (k, i) in &column_names {
                map.insert(k.as_str(), db::mysql::mysql_value_to_lua(row[*i].clone()));
            }

            result.push(map.into());
        }

        Ok(result)
    }))
}

fn format_lua(out: &mut String, x: &AnyLuaValue) {
    match *x {
        AnyLuaValue::LuaNil => out.push_str("null"),
        AnyLuaValue::LuaString(ref x) => out.push_str(&format!("{:?}", x)),
        AnyLuaValue::LuaNumber(ref x) => out.push_str(&format!("{:?}", x)),
        AnyLuaValue::LuaAnyString(ref x) => out.push_str(&format!("{:?}", x.0)),
        AnyLuaValue::LuaBoolean(ref x) => out.push_str(&format!("{:?}", x)),
        AnyLuaValue::LuaArray(ref x) => {
            out.push_str("{");
            let mut first = true;

            for &(ref k, ref v) in x {
                if !first {
                    out.push_str(", ");
                }

                let mut key = String::new();
                format_lua(&mut key, &k);

                let mut value = String::new();
                format_lua(&mut value, &v);

                out.push_str(&format!("{}: {}", key, value));

                first = false;
            }
            out.push_str("}");
        },
        AnyLuaValue::LuaOther => out.push_str("LuaOther"),
    }
}

pub fn print(lua: &mut hlua::Lua, _: State) {
    // this function doesn't print to the terminal safely
    // only use this for debugging
    lua.set("print", hlua::function1(move |val: AnyLuaValue| {
        // println!("{:?}", val);
        let mut out = String::new();
        format_lua(&mut out, &val);
        println!("{}", out);
    }))
}

pub fn rand(lua: &mut hlua::Lua, _: State) {
    lua.set("rand", hlua::function2(move |min: u32, max: u32| -> u32 {
        let mut rng = rand::thread_rng();
        (rng.next_u32() + min) % max
    }))
}

pub fn randombytes(lua: &mut hlua::Lua, _: State) {
    lua.set("randombytes", hlua::function1(move |num: u32| -> AnyLuaValue {
        let mut x = vec![0; num as usize];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(x.as_mut_slice());
        lua_bytes(&x)
    }))
}

pub fn sha1(lua: &mut hlua::Lua, state: State) {
    lua.set("sha1", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map(|bytes| lua_bytes(&sha1::Sha1::digest(&bytes)))
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sha2_256(lua: &mut hlua::Lua, state: State) {
    lua.set("sha2_256", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map(|bytes| lua_bytes(&sha2::Sha256::digest(&bytes)))
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sha2_512(lua: &mut hlua::Lua, state: State) {
    lua.set("sha2_512", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map(|bytes| lua_bytes(&sha2::Sha512::digest(&bytes)))
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sha3_256(lua: &mut hlua::Lua, state: State) {
    lua.set("sha3_256", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map(|bytes| lua_bytes(&sha3::Sha3_256::digest(&bytes)))
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sha3_512(lua: &mut hlua::Lua, state: State) {
    lua.set("sha3_512", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map(|bytes| lua_bytes(&sha3::Sha3_512::digest(&bytes)))
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sleep(lua: &mut hlua::Lua, _: State) {
    lua.set("sleep", hlua::function1(move |n: i32| {
        thread::sleep(Duration::from_secs(n as u64));
        0
    }))
}
