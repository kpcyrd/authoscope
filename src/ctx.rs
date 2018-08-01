use hlua::{self, AnyLuaValue};
use errors::{Result, Error};
use runtime;

use std::fs::File;
use std::sync::{Arc, Mutex};
use std::io::prelude::*;
use std::collections::HashMap;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use http::{HttpSession,
           HttpRequest,
           RequestOptions};
use config::Config;
use mysql;
use sockets::Socket;


#[derive(Debug, Clone)]
pub struct State {
    config: Arc<Config>,
    error: Arc<Mutex<Option<Error>>>,
    http_sessions: Arc<Mutex<HashMap<String, HttpSession>>>,
    mysql_sessions: Arc<Mutex<HashMap<String, Arc<Mutex<mysql::Conn>>>>>,
    socket_sessions: Arc<Mutex<HashMap<String, Arc<Mutex<Socket>>>>>,
}

impl State {
    pub fn new(config: Arc<Config>) -> State {
        State {
            config,
            error: Arc::new(Mutex::new(None)),
            http_sessions: Arc::new(Mutex::new(HashMap::new())),
            mysql_sessions: Arc::new(Mutex::new(HashMap::new())),
            socket_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn last_error(&self) -> Option<String> {
        let lock = self.error.lock().unwrap();
        lock.as_ref().map(|err| err.to_string())
    }

    pub fn clear_error(&self) {
        let mut lock = self.error.lock().unwrap();
        *lock = None;
    }

    pub fn set_error(&self, err: Error) -> Error {
        let mut mtx = self.error.lock().unwrap();
        let cp = err.to_string();
        *mtx = Some(err);
        cp.into()
    }

    fn random_id(&self) -> String {
        thread_rng().sample_iter(&Alphanumeric).take(16).collect()
    }

    pub fn register_in_jar(&self, session: &str, cookies: Vec<(String, String)>) {
        let mut mtx = self.http_sessions.lock().unwrap();
        if let Some(session) = mtx.get_mut(session) {
            session.cookies.register_in_jar(cookies);
        }
    }

    pub fn http_mksession(&self) -> String {
        let mut mtx = self.http_sessions.lock().unwrap();
        let (id, session) = HttpSession::new();
        mtx.insert(id.clone(), session);
        id
    }

    pub fn http_request(&self, session_id: &str, method: String, url: String, options: RequestOptions) -> HttpRequest {
        let mtx = self.http_sessions.lock().unwrap();
        let session = mtx.get(session_id).expect("invalid session reference"); // TODO

        HttpRequest::new(&self.config, &session, method, url, options)
    }

    pub fn mysql_register(&self, sock: mysql::Conn) -> String {
        let mut mtx = self.mysql_sessions.lock().unwrap();
        let id = self.random_id();

        let sock = Arc::new(Mutex::new(sock));
        mtx.insert(id.clone(), sock);

        id
    }

    pub fn mysql_session(&self, id: &str) -> Arc<Mutex<mysql::Conn>> {
        let mtx = self.mysql_sessions.lock().unwrap();
        let sock = mtx.get(id).expect("invalid session reference"); // TODO
        sock.clone()
    }

    pub fn sock_connect(&self, host: &str, port: u16) -> Result<String> {
        let mut mtx = self.socket_sessions.lock().unwrap();
        let id = self.random_id();

        let sock = Socket::connect(host, port)?;
        mtx.insert(id.clone(), Arc::new(Mutex::new(sock)));

        Ok(id)
    }

    pub fn get_sock(&self, id: &str)-> Arc<Mutex<Socket>> {
        let mtx = self.socket_sessions.lock().unwrap();
        let sock = mtx.get(id).expect("invalid session reference"); // TODO
        sock.clone()
    }
}


#[derive(Debug, Clone)]
pub struct Script {
    descr: String,
    code: String,
    config: Arc<Config>,
}

impl Script {
    pub fn load(path: &str, config: Arc<Config>) -> Result<Script> {
        let mut file = File::open(path)?;
        Script::load_from(&mut file, config)
    }

    pub fn load_from<R: Read>(mut src: R, config: Arc<Config>) -> Result<Script> {
        let mut code = String::new();
        src.read_to_string(&mut code)?;

        let (mut lua, _) = Script::ctx(&config);
        lua.execute::<()>(&code)?;

        let descr = {
            let descr: Result<_> = lua.get("descr").ok_or_else(|| "descr undefined".into());
            let descr: hlua::StringInLua<_> = descr?;
            (*descr).to_owned()
        };

        {
            let verify: Result<_> = lua.get("verify").ok_or_else(|| "verify undefined".into());
            let _: hlua::LuaFunction<_> = verify?;
        };

        Ok(Script {
            descr,
            code,
            config,
        })
    }

    fn ctx<'a>(config: &Arc<Config>) -> (hlua::Lua<'a>, State) {
        let mut lua = hlua::Lua::new();
        lua.open_string();
        let state = State::new(config.clone());

        runtime::base64_decode(&mut lua, state.clone());
        runtime::base64_encode(&mut lua, state.clone());
        runtime::bcrypt(&mut lua, state.clone());
        runtime::bcrypt_verify(&mut lua, state.clone());
        runtime::clear_err(&mut lua, state.clone());
        runtime::execve(&mut lua, state.clone());
        runtime::hex(&mut lua, state.clone());
        runtime::hmac_md5(&mut lua, state.clone());
        runtime::hmac_sha1(&mut lua, state.clone());
        runtime::hmac_sha2_256(&mut lua, state.clone());
        runtime::hmac_sha2_512(&mut lua, state.clone());
        runtime::hmac_sha3_256(&mut lua, state.clone());
        runtime::hmac_sha3_512(&mut lua, state.clone());
        runtime::html_select(&mut lua, state.clone());
        runtime::html_select_list(&mut lua, state.clone());
        runtime::http_basic_auth(&mut lua, state.clone()); // TODO: deprecate?
        runtime::http_mksession(&mut lua, state.clone());
        runtime::http_request(&mut lua, state.clone());
        runtime::http_send(&mut lua, state.clone());
        runtime::json_decode(&mut lua, state.clone());
        runtime::json_encode(&mut lua, state.clone());
        runtime::last_err(&mut lua, state.clone());
        runtime::ldap_bind(&mut lua, state.clone());
        runtime::ldap_escape(&mut lua, state.clone());
        runtime::ldap_search_bind(&mut lua, state.clone());
        runtime::md5(&mut lua, state.clone());
        runtime::mysql_connect(&mut lua, state.clone());
        runtime::mysql_query(&mut lua, state.clone());
        runtime::print(&mut lua, state.clone());
        runtime::rand(&mut lua, state.clone());
        runtime::randombytes(&mut lua, state.clone());
        runtime::sha1(&mut lua, state.clone());
        runtime::sha2_256(&mut lua, state.clone());
        runtime::sha2_512(&mut lua, state.clone());
        runtime::sha3_256(&mut lua, state.clone());
        runtime::sha3_512(&mut lua, state.clone());
        runtime::sleep(&mut lua, state.clone());
        runtime::sock_connect(&mut lua, state.clone());
        runtime::sock_send(&mut lua, state.clone());
        runtime::sock_recv(&mut lua, state.clone());
        runtime::sock_sendline(&mut lua, state.clone());
        runtime::sock_recvline(&mut lua, state.clone());
        runtime::sock_recvall(&mut lua, state.clone());
        runtime::sock_recvline_contains(&mut lua, state.clone());
        runtime::sock_recvline_regex(&mut lua, state.clone());
        runtime::sock_recvn(&mut lua, state.clone());
        runtime::sock_recvuntil(&mut lua, state.clone());
        runtime::sock_sendafter(&mut lua, state.clone());
        runtime::sock_newline(&mut lua, state.clone());

        (lua, state)
    }

    #[inline]
    pub fn descr(&self) -> &str {
        self.descr.as_str()
    }

    /*
    #[inline]
    pub fn code(&self) -> &str {
        self.code.as_str()
    }
    */

    pub fn run_once(&self, user: AnyLuaValue, password: AnyLuaValue) -> Result<bool> {
        debug!("executing {:?} with {:?}:{:?}", self.descr(), user, password);

        let (mut lua, state) = Script::ctx(&self.config);
        lua.execute::<()>(&self.code)?;

        let verify: Result<_> = lua.get("verify").ok_or_else(|| "verify undefined".into());
        let mut verify: hlua::LuaFunction<_> = verify?;

        let result: hlua::AnyLuaValue = match verify.call_with_args((user, password)) {
            Ok(res) => res,
            Err(err) => {
                let err = format!("execution failed: {:?}", err);
                return Err(err.into())
            },
        };

        if let Some(err) = state.error.lock().unwrap().take() {
            return Err(err);
        }

        use hlua::AnyLuaValue::*;
        match result {
            LuaBoolean(x) => Ok(x),
            LuaString(x) => Err(format!("error: {:?}", x).into()),
            x => Err(format!("lua returned wrong type: {:?}", x).into()),
        }
    }

    #[inline]
    pub fn run_creds(&self, user: &str, password: &str) -> Result<bool> {
        let user = AnyLuaValue::LuaString(user.to_string());
        let password = AnyLuaValue::LuaString(password.to_string());
        self.run_once(user, password)
    }

    #[inline]
    pub fn run_enum(&self, user: &str) -> Result<bool> {
        let user = AnyLuaValue::LuaString(user.to_string());
        let password = AnyLuaValue::LuaNil;
        self.run_once(user, password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_config() -> Arc<Config> {
        Arc::new(Config::default())
    }

    #[test]
    fn verify_false() {
        let script = Script::load_from(r#"
        descr = "verify_false"

        function verify(user, password)
            return false
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("foo", "bar").expect("test script failed");
        assert!(!result);
    }

    #[test]
    fn verify_true() {
        let script = Script::load_from(r#"
        descr = "verify_false"

        function verify(user, password)
            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("foo", "bar").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_record_error() {
        let script = Script::load_from(r#"
        descr = "json"

        function verify(user, password)
            json_decode("{{{{{{{{{{{{{{{{{{")
            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x");
        assert!(result.is_err());
    }

    #[test]
    fn verify_clear_recorded_error() {
        let script = Script::load_from(r#"
        descr = "json"

        function verify(user, password)
            json_decode("{{{{{{{{{{{{{{{{{{")
            clear_err()
            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_sleep() {
        let script = Script::load_from(r#"
        descr = "slow script"

        function verify(user, password)
            sleep(1)
            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("foo", "bar").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_basic_auth_correct() {
        let script = Script::load_from(r#"
        descr = "basic auth httpbin.org"

        function verify(user, password)
            return http_basic_auth("https://httpbin.org/basic-auth/foo/buzz", user, password)
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("foo", "buzz").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_basic_auth_incorrect() {
        let script = Script::load_from(r#"
        descr = "basic auth httpbin.org"

        function verify(user, password)
            return http_basic_auth("https://httpbin.org/basic-auth/foo/buzz", user, password)
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("invalid", "wrong").expect("test script failed");
        assert!(!result);
    }

    #[test]
    fn verify_hex() {
        let script = Script::load_from(r#"
        descr = "hex test"

        function verify(user, password)
            x = hex({0x6F, 0x68, 0x61, 0x69, 0x0A, 0x00})
            return x == "6f6861690a00"
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_hex_empty() {
        let script = Script::load_from(r#"
        descr = "hex test"

        function verify(user, password)
            x = hex({})
            return x == ""
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_json_encode() {
        let script = Script::load_from(r#"
        descr = "json"

        function verify(user, password)
            json_encode({
                hello="world",
                almost_one=0.9999,
                list={1,3,3,7},
                data={
                    user=user,
                    password=password,
                    empty=nil
                }
            })
            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_json_encode_decode() {
        let script = Script::load_from(r#"
        descr = "json"

        function verify(user, password)
            x = json_encode({
                hello="world",
                almost_one=0.9999,
                list={1,3,3,7},
                data={
                    user=user,
                    password=password,
                    empty=nil
                }
            })
            json_decode(x)

            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_json_decode_valid() {
        let script = Script::load_from(r#"
        descr = "json"

        function verify(user, password)
            json_decode("{\"almost_one\":0.9999,\"data\":{\"password\":\"fizz\",\"user\":\"bar\"},\"hello\":\"world\",\"list\":[1,3,3,7]}")
            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_json_decode_invalid() {
        let script = Script::load_from(r#"
        descr = "json"

        function verify(user, password)
            json_decode("{\"almost_one\":0.9999,\"data\":{\"password\":\"fizz\",\"user\":\"bar\"}}}}}}}}}")
            return true
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x");
        assert!(result.is_err());
    }

    #[test]
    fn verify_hmac_md5() {
        let script = Script::load_from(r#"
        descr = "hmac_md5"

        function verify(user, password)
            x = hex(hmac_md5("foo", "bar"))
            -- print('md5: ' .. x)
            return x == "0c7a250281315ab863549f66cd8a3a53"
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_hmac_sha1() {
        let script = Script::load_from(r#"
        descr = "hmac_sha1"

        function verify(user, password)
            x = hex(hmac_sha1("foo", "bar"))
            -- print('sha1: ' .. x)
            return x == "46b4ec586117154dacd49d664e5d63fdc88efb51"
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_hmac_sha2_256() {
        let script = Script::load_from(r#"
        descr = "hmac_sha2_256"

        function verify(user, password)
            x = hex(hmac_sha2_256("foo", "bar"))
            -- print('sha2_256: ' .. x)
            return x == "f9320baf0249169e73850cd6156ded0106e2bb6ad8cab01b7bbbebe6d1065317"
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_hmac_sha2_512() {
        let script = Script::load_from(r#"
        descr = "hmac_sha2_512"

        function verify(user, password)
            x = hex(hmac_sha2_512("foo", "bar"))
            -- print('sha2_512: ' .. x)
            return x == "114682914c5d017dfe59fdc804118b56a3a652a0b8870759cf9e792ed7426b08197076bf7d01640b1b0684df79e4b67e37485669e8ce98dbab60445f0db94fce"
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_hmac_sha3_256() {
        let script = Script::load_from(r#"
        descr = "hmac_sha3_256"

        function verify(user, password)
            x = hex(hmac_sha3_256("foo", "bar"))
            -- print('sha3_256: ' .. x)
            return x == "a7dc3fbbd45078239f0cb321e6902375d22b505f2c48722eb7009e7da2574893"
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_hmac_sha3_512() {
        let script = Script::load_from(r#"
        descr = "hmac_sha3_512"

        function verify(user, password)
            x = hex(hmac_sha3_512("foo", "bar"))
            -- print('sha3_512: ' .. x)
            return x == "2da91b8227d106199fd06c5d8a6752796cf3c84dde5a427bd2aca384f0cffc19997e2584ed15c55542c2cb8918b987e2bcd9e77a9f3fdbb4dbea8a3d0136da2f"
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "x").expect("test script failed");
        assert!(result);
    }

    #[test]
    fn verify_bcrypt_verify() {
        let script = Script::load_from(r#"
        descr = "bcrypt_verify"

        function verify(user, password)
            return bcrypt_verify(password, "$2a$12$ByUlHCHx3rxMsdQONpuFbulQqut6GQ/84I5EAUkCqTTI07JA7wUju")
        end
        "#.as_bytes(), empty_config()).unwrap();

        let result = script.run_creds("x", "hunter2").expect("test script failed");
        assert!(result);
    }
}
