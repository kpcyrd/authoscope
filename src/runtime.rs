use hlua;

use reqwest;
use ldap3;
use mysql;
use rand;
use rand::Rng;

use std::thread;
use std::time::Duration;
use std::process::Command;


pub fn execve(lua: &mut hlua::Lua) {
    lua.set("execve", hlua::function2(move |prog: String, args: Vec<hlua::AnyLuaValue>| -> i32 {
        let args: Vec<_> = args.into_iter()
                    .flat_map(|x| match x {
                        hlua::AnyLuaValue::LuaString(x) => Some(x),
                        _ => None, // TODO: error
                    })
                    .collect();

        let status = Command::new(prog)
                    .args(&args)
                    .status()
                    .expect("TODO: failed to spawn is fatal");

        status.code().expect("TODO: termination by signal is fatal")
    }))
}

pub fn http_basic_auth(lua: &mut hlua::Lua) {
    lua.set("http_basic_auth", hlua::function3(move |url: String, user: String, password: String| -> bool {
        let client = reqwest::Client::new();

        let response = client.get(&url)
                             .basic_auth(user, Some(password))
                             .send().expect("TODO: http error is fatal");

        // println!("{:?}", response);
        // println!("{:?}", response.headers().get_raw("www-authenticate"));
        // println!("{:?}", response.status());

        response.headers().get_raw("www-authenticate").is_none() &&
            response.status() != reqwest::StatusCode::Unauthorized
    }))
}

pub fn ldap_bind(lua: &mut hlua::Lua) {
    lua.set("ldap_bind", hlua::function3(move |url: String, dn: String, password: String| -> bool {
        let sock = ldap3::LdapConn::new(&url).expect("TODO: ldap error is fatal");

        let result = sock.simple_bind(&dn, &password).expect("TODO: ldap error is fatal");
        // println!("{:?}", result);

        result.success().is_ok()
    }))
}

pub fn ldap_escape(lua: &mut hlua::Lua) {
    lua.set("ldap_escape", hlua::function1(move |s: String| -> String {
        ldap3::dn_escape(s).to_string()
    }))
}

pub fn mysql_connect(lua: &mut hlua::Lua) {
    lua.set("mysql_connect", hlua::function4(move |host: String, port: u16, user: String, password: String| -> bool {
        let mut builder = mysql::OptsBuilder::new();
        builder.ip_or_hostname(Some(host))
               .tcp_port(port)
               .prefer_socket(false)
               .user(Some(user))
               .pass(Some(password));

        match mysql::Conn::new(builder) {
            Ok(_) => true,
            Err(_err) => {
                // TODO: err
                // println!("{:?}", _err);
                false
            },
        }
    }))
}

pub fn rand(lua: &mut hlua::Lua) {
    lua.set("rand", hlua::function2(move |min: u32, max: u32| -> u32 {
        let mut rng = rand::thread_rng();
        (rng.next_u32() + min) % max
    }))
}

pub fn sleep(lua: &mut hlua::Lua) {
    lua.set("sleep", hlua::function1(move |n: i32| {
        thread::sleep(Duration::from_secs(n as u64));
        0
    }))
}
