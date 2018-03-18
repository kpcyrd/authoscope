use hlua;
use hlua::AnyLuaValue;
use hlua::AnyLuaValue::LuaString;
use errors::{Result, ResultExt};

use reqwest;
use ldap3;
use mysql;
use rand;
use rand::Rng;

use std::thread;
use std::time::Duration;
use std::process::Command;
use ctx::State;


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

pub fn ldap_bind(lua: &mut hlua::Lua, state: State) {
    lua.set("ldap_bind", hlua::function3(move |url: String, dn: String, password: String| -> Result<bool> {
        let sock = match ldap3::LdapConn::new(&url)
                        .chain_err(|| "ldap connection failed") {
            Ok(sock) => sock,
            Err(err) => return Err(state.set_error(err)),
        };

        let result = match sock.simple_bind(&dn, &password)
                            .chain_err(|| "fatal error during simple_bind: {:?}") {
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

pub fn mysql_connect(lua: &mut hlua::Lua, _state: State) {
    lua.set("mysql_connect", hlua::function4(move |host: String, port: u16, user: String, password: String| -> Result<bool> {
        let mut builder = mysql::OptsBuilder::new();
        builder.ip_or_hostname(Some(host))
               .tcp_port(port)
               .prefer_socket(false)
               .user(Some(user))
               .pass(Some(password));

        match mysql::Conn::new(builder) {
            Ok(_) => Ok(true),
            Err(_err) => {
                // TODO: err
                // println!("{:?}", _err);
                Ok(false)
            },
        }
    }))
}

pub fn rand(lua: &mut hlua::Lua, _: State) {
    lua.set("rand", hlua::function2(move |min: u32, max: u32| -> u32 {
        let mut rng = rand::thread_rng();
        (rng.next_u32() + min) % max
    }))
}

pub fn sleep(lua: &mut hlua::Lua, _: State) {
    lua.set("sleep", hlua::function1(move |n: i32| {
        thread::sleep(Duration::from_secs(n as u64));
        0
    }))
}
