use crate::hlua;
use crate::hlua::{AnyLuaValue, AnyHashableLuaValue, AnyLuaString};
use crate::hlua::AnyLuaValue::LuaString;
use crate::structs::LuaMap;
use crate::errors::*;
use crate::json;
use crate::db;

use digest::{Digest, Update, BlockInput, FixedOutput, Reset};
use digest::generic_array::ArrayLength;
use hmac::{Hmac, NewMac, Mac};
use mysql::prelude::Queryable;

use reqwest::header::WWW_AUTHENTICATE;
use rand::RngCore;

use std::thread;
use std::time::Duration;
use std::process::Command;
use std::collections::HashMap;
use crate::ctx::State;
use crate::http::HttpRequest;
use crate::http::RequestOptions;
use crate::html;


fn byte_array(bytes: AnyLuaValue) -> Result<Vec<u8>> {
    match bytes {
        AnyLuaValue::LuaAnyString(bytes) => Ok(bytes.0),
        AnyLuaValue::LuaString(bytes) => Ok(bytes.into_bytes()),
        AnyLuaValue::LuaArray(bytes) => {
            Ok(bytes.into_iter()
                .map(|num| match num.1 {
                    AnyLuaValue::LuaNumber(num) if (0.0..=255.0).contains(&num) && (num % 1.0 == 0.0) =>
                            Ok(num as u8),
                    AnyLuaValue::LuaNumber(num) =>
                            Err(format_err!("number is out of range: {:?}", num)),
                    _ => Err(format_err!("unexpected type: {:?}", num)),
                })
                .collect::<Result<_>>()?)
        },
        _ => Err(format_err!("Invalid type: {:?}", bytes)),
    }
}

fn lua_bytes(bytes: &[u8]) -> AnyLuaValue {
    let bytes = AnyLuaString(bytes.to_vec());
    AnyLuaValue::LuaAnyString(bytes)
}

pub fn base64_decode(lua: &mut hlua::Lua, state: State) {
    lua.set("base64_decode", hlua::function1(move |bytes: String| -> Result<AnyLuaValue> {
        base64::decode(&bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&bytes))
    }))
}

pub fn base64_encode(lua: &mut hlua::Lua, state: State) {
    lua.set("base64_encode", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| base64::encode(&bytes))
    }))
}

pub fn bcrypt(lua: &mut hlua::Lua, state: State) {
    lua.set("bcrypt", hlua::function2(move |password: String, cost: u32| -> Result<String> {
        bcrypt::hash(&password, cost)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn bcrypt_verify(lua: &mut hlua::Lua, state: State) {
    lua.set("bcrypt_verify", hlua::function2(move |password: String, hashed: String| -> Result<bool> {
        bcrypt::verify(&password, &hashed)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn clear_err(lua: &mut hlua::Lua, state: State) {
    lua.set("clear_err", hlua::function0(move || {
        state.clear_error()
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
                        .context("Failed to spawn program") {
            Ok(status) => status,
            Err(err) => return Err(state.set_error(err)),
        };

        let code = match status.code() {
            Some(code) => code,
            None => return Err(state.set_error(format_err!("Process didn't return exit code"))),
        };

        Ok(code)
    }))
}

pub fn hex(lua: &mut hlua::Lua, state: State) {
    lua.set("hex", hlua::function1(move |bytes: AnyLuaValue| -> Result<String> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| {
                let mut out = String::new();

                for b in bytes {
                    out += &format!("{:02x}", b);
                }

                out
            })
    }))
}

fn hmac<D>(secret: AnyLuaValue, msg: AnyLuaValue) -> Result<AnyLuaValue>
    where
        D: Update + BlockInput + FixedOutput + Reset + Default + Clone,
        D::BlockSize: ArrayLength<u8> + Clone,
        D::OutputSize: ArrayLength<u8>,
{
    let secret = byte_array(secret)?;
    let msg = byte_array(msg)?;

    let mut mac = match Hmac::<D>::new_from_slice(&secret) {
        Ok(mac) => mac,
        Err(_) => bail!("Invalid key length"),
    };
    mac.update(&msg);
    let result = mac.finalize();
    Ok(lua_bytes(&result.into_bytes()))
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
            .map_err(|err| state.set_error(err))
            .map(|x| x.into())
    }))
}

pub fn html_select_list(lua: &mut hlua::Lua, state: State) {
    lua.set("html_select_list", hlua::function2(move |html: String, selector: String| -> Result<Vec<AnyLuaValue>> {
        html::html_select_list(&html, &selector)
            .map_err(|err| state.set_error(err))
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }))
}

pub fn http_basic_auth(lua: &mut hlua::Lua, state: State) {
    lua.set("http_basic_auth", hlua::function3(move |url: String, user: String, password: String| -> Result<bool> {
        let client = reqwest::blocking::Client::new();

        client.get(&url)
            .basic_auth(user, Some(password))
            .send()
            .context("http request failed")
            .map_err(|err| state.set_error(err))
            .map(|response| {
                info!("http_basic_auth: {:?}", response);
                response.headers().get(WWW_AUTHENTICATE).is_none() &&
                    response.status() != reqwest::StatusCode::UNAUTHORIZED
            })
    }))
}

pub fn http_mksession(lua: &mut hlua::Lua, state: State) {
    lua.set("http_mksession", hlua::function0(move || -> String {
        state.http_mksession()
    }))
}

pub fn http_request(lua: &mut hlua::Lua, state: State) {
    lua.set("http_request", hlua::function4(move |session: String, method: String, url: String, options: AnyLuaValue| -> Result<AnyLuaValue> {
        RequestOptions::try_from(options)
            .context("Invalid request options")
            .map_err(|err| state.set_error(err))
            .map(|options| {
                state.http_request(&session, method, url, options).into()
            })
    }))
}

pub fn http_send(lua: &mut hlua::Lua, state: State) {
    lua.set("http_send", hlua::function1(move |request: AnyLuaValue| -> Result<HashMap<AnyHashableLuaValue, AnyLuaValue>> {
        let req = match HttpRequest::try_from(request)
                                .context("Invalid http request object") {
            Ok(req) => req,
            Err(err) => return Err(state.set_error(err)),
        };

        req.send(&state)
            .map_err(|err| state.set_error(err))
            .map(|resp| resp.into())
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
        let mut sock = match ldap3::LdapConn::new(&url)
                        .context("ldap connection failed") {
            Ok(sock) => sock,
            Err(err) => return Err(state.set_error(err)),
        };

        sock.simple_bind(&dn, &password)
            .context("Fatal error during simple_bind")
            .map_err(|err| state.set_error(err))
            .map(|result| {
                debug!("ldap_bind: {:?}", result);
                result.success().is_ok()
            })
    }))
}

pub fn ldap_escape(lua: &mut hlua::Lua, _: State) {
    lua.set("ldap_escape", hlua::function1(move |s: String| -> String {
        ldap3::dn_escape(s).to_string()
    }))
}

pub fn ldap_search_bind(lua: &mut hlua::Lua, state: State) {
    lua.set("ldap_search_bind", hlua::function6(move |url: String, search_user: String, search_pw: String, base_dn: String, user: String, password: String| -> Result<bool> {
        let mut sock = ldap3::LdapConn::new(&url)
            .context("ldap connection failed")
            .map_err(|err| state.set_error(err))?;


        let result = sock.simple_bind(&search_user, &search_pw)
            .context("Fatal error during simple_bind with search user")
            .map_err(|err| state.set_error(err))?;

        if result.success().is_err() {
            return Err(state.set_error(format_err!("Login with search user failed")));
        }

        let search = format!("uid={}", ldap3::dn_escape(user));
        let result = sock.search(&base_dn, ldap3::Scope::Subtree, &search, vec!["*"])
            .context("Fatal error during ldap search")
            .map_err(|err| state.set_error(err))?;

        let entries = result.success()
            .context("ldap search failed")
            .map(|result| result.0)
            .map_err(|err| state.set_error(err))?;

        // take the first result
        if let Some(entry) = entries.into_iter().next() {
            let entry = ldap3::SearchEntry::construct(entry);

            // we got the DN, try to login
            let result = sock.simple_bind(&entry.dn, &password)
                .context("Fatal error during simple_bind")
                .map_err(|err| state.set_error(err))?;

            // println!("{:?}", result);

            Ok(result.success().is_ok())
        } else {
            Ok(false)
        }
    }))
}

pub fn md5(lua: &mut hlua::Lua, state: State) {
    lua.set("md5", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&md5::Md5::digest(&bytes)))
    }))
}

pub fn mysql_connect(lua: &mut hlua::Lua, state: State) {
    lua.set("mysql_connect", hlua::function4(move |host: String, port: u16, user: String, password: String| -> Result<String> {
        let builder = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(host))
            .tcp_port(port)
            .prefer_socket(false)
            .user(Some(user))
            .pass(Some(password));

        mysql::Conn::new(builder)
            .map_err(|err| state.set_error(err))
            .map(|sock| state.mysql_register(sock))
    }))
}

pub fn mysql_query(lua: &mut hlua::Lua, state: State) {
    lua.set("mysql_query", hlua::function3(move |session: String, query: String, params: HashMap<AnyHashableLuaValue, AnyLuaValue>| -> Result<Vec<AnyLuaValue>> {
        let params = LuaMap::from(params);

        let sock = state.mysql_session(&session);
        let mut sock = sock.lock().unwrap();
        let rows = sock.exec_iter(query, params)
            .context("Failed to execute query")
            .map_err(|err| state.set_error(err))?;

        let columns = rows.columns();
        let mut columns_results = Vec::new();
        for column in columns.as_ref() {
            let column = column.name_str().to_string();
            columns_results.push(column);
        }

        let mut result = Vec::new();
        for row in rows {
            let row = row?; // TODO: handle error

            let mut map = LuaMap::new();
            for (i, column) in columns_results.iter().enumerate() {
                map.insert(column, db::mysql::mysql_value_to_lua(row[i].clone()));
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
            out.push('{');
            let mut first = true;

            for &(ref k, ref v) in x {
                if !first {
                    out.push_str(", ");
                }

                let mut key = String::new();
                format_lua(&mut key, k);

                let mut value = String::new();
                format_lua(&mut value, v);

                out.push_str(&format!("{}: {}", key, value));

                first = false;
            }
            out.push('}');
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
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha1::Sha1::digest(&bytes)))
    }))
}

pub fn sha2_256(lua: &mut hlua::Lua, state: State) {
    lua.set("sha2_256", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha2::Sha256::digest(&bytes)))
    }))
}

pub fn sha2_512(lua: &mut hlua::Lua, state: State) {
    lua.set("sha2_512", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha2::Sha512::digest(&bytes)))
    }))
}

pub fn sha3_256(lua: &mut hlua::Lua, state: State) {
    lua.set("sha3_256", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha3::Sha3_256::digest(&bytes)))
    }))
}

pub fn sha3_512(lua: &mut hlua::Lua, state: State) {
    lua.set("sha3_512", hlua::function1(move |bytes: AnyLuaValue| -> Result<AnyLuaValue> {
        byte_array(bytes)
            .map_err(|err| state.set_error(err))
            .map(|bytes| lua_bytes(&sha3::Sha3_512::digest(&bytes)))
    }))
}

pub fn sleep(lua: &mut hlua::Lua, _: State) {
    lua.set("sleep", hlua::function1(move |n: i32| {
        thread::sleep(Duration::from_secs(n as u64));
        0
    }))
}

pub fn sock_connect(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_connect", hlua::function3(move |host: String, port: u16, _settings: AnyLuaValue| -> Result<String> {
        state.sock_connect(&host, port)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn sock_send(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_send", hlua::function2(move |sock: String, bytes: AnyLuaValue| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        sock.send(&bytes)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_recv(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_recv", hlua::function1(move |sock: String| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recv()
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_sendline(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_sendline", hlua::function2(move |sock: String, line: String| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        sock.sendline(&line)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_recvline(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_recvline", hlua::function1(move |sock: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline()
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvall(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_recvall", hlua::function1(move |sock: String| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recvall()
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_recvline_contains(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_recvline_contains", hlua::function2(move |sock: String, needle: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline_contains(&needle)
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvline_regex(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_recvline_regex", hlua::function2(move |sock: String, regex: String| -> Result<String> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let line = sock.recvline_regex(&regex)
            .map_err(|err| state.set_error(err))?;

        Ok(line)
    }))
}

pub fn sock_recvn(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_recvn", hlua::function2(move |sock: String, n: u32| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let bytes = sock.recvn(n)
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_recvuntil(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_recvuntil", hlua::function2(move |sock: String, delim: AnyLuaValue| -> Result<AnyLuaValue> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let delim = byte_array(delim)
            .map_err(|err| state.set_error(err))?;

        let bytes = sock.recvuntil(&delim)
            .map_err(|err| state.set_error(err))?;

        Ok(lua_bytes(&bytes))
    }))
}

pub fn sock_sendafter(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_sendafter", hlua::function3(move |sock: String, delim: AnyLuaValue, bytes: AnyLuaValue| -> Result<()> {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        let delim = byte_array(delim)
            .map_err(|err| state.set_error(err))?;

        let bytes = byte_array(bytes)
            .map_err(|err| state.set_error(err))?;

        sock.sendafter(&delim, &bytes)
            .map_err(|err| state.set_error(err))?;

        Ok(())
    }))
}

pub fn sock_newline(lua: &mut hlua::Lua, state: State) {
    lua.set("sock_newline", hlua::function2(move |sock: String, newline: String| {
        let sock = state.get_sock(&sock);
        let mut sock = sock.lock().unwrap();

        sock.newline(newline);
    }))
}
