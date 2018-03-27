use hlua;
use errors::{Result, Error};
use runtime;

use std::fs::File;
use std::sync::{Arc, Mutex};
use std::io::prelude::*;


#[derive(Debug, Clone)]
pub struct State {
    error: Arc<Mutex<Option<Error>>>,
}

impl State {
    pub fn new() -> State {
        State {
            error: Arc::new(Mutex::new(None)),
        }
    }

    pub fn last_error(&self) -> Option<String> {
        let lock = self.error.lock().unwrap();
        match *lock {
            Some(ref err) => Some(err.to_string()),
            None => None,
        }
    }

    pub fn set_error(&self, err: Error) -> Error {
        let mut mtx = self.error.lock().unwrap();
        let cp = err.to_string();
        *mtx = Some(err);
        return cp.into();
    }
}


#[derive(Debug, Clone)]
pub struct Script {
    descr: String,
    code: String,
}

impl Script {
    pub fn load(path: &str) -> Result<Script> {
        let mut file = File::open(path)?;
        Script::load_from(&mut file)
    }

    pub fn load_from<R: Read>(mut src: R) -> Result<Script> {
        let mut code = String::new();
        src.read_to_string(&mut code)?;

        let (mut lua, _) = Script::ctx();
        lua.execute::<()>(&code)?;

        let descr = {
            let descr: Result<_> = lua.get("descr").ok_or("descr undefined".into());
            let descr: hlua::StringInLua<_> = descr?;
            (*descr).to_owned()
        };

        {
            let verify: Result<_> = lua.get("verify").ok_or("verify undefined".into());
            let _: hlua::LuaFunction<_> = verify?;
        };

        Ok(Script {
            descr,
            code,
        })
    }

    fn ctx<'a>() -> (hlua::Lua<'a>, State) {
        let mut lua = hlua::Lua::new();
        let state = State::new();

        runtime::execve(&mut lua, state.clone());
        runtime::hex(&mut lua, state.clone());
        runtime::http_basic_auth(&mut lua, state.clone());
        runtime::json_decode(&mut lua, state.clone());
        runtime::json_encode(&mut lua, state.clone());
        runtime::last_err(&mut lua, state.clone());
        runtime::ldap_bind(&mut lua, state.clone());
        runtime::ldap_escape(&mut lua, state.clone());
        runtime::ldap_search_bind(&mut lua, state.clone());
        runtime::mysql_connect(&mut lua, state.clone());
        runtime::print(&mut lua, state.clone());
        runtime::rand(&mut lua, state.clone());
        runtime::sleep(&mut lua, state.clone());

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

    pub fn run_once(&self, user: &str, password: &str) -> Result<bool> {
        let (mut lua, state) = Script::ctx();
        lua.execute::<()>(&self.code)?;

        let verify: Result<_> = lua.get("verify").ok_or("verify undefined".into());
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_false() {
        let script = Script::load_from(r#"
        descr = "verify_false"

        function verify(user, password)
            return false
        end
        "#.as_bytes()).unwrap();

        let result = script.run_once("foo", "bar").unwrap();
        assert!(!result);
    }

    #[test]
    fn verify_true() {
        let script = Script::load_from(r#"
        descr = "verify_false"

        function verify(user, password)
            return true
        end
        "#.as_bytes()).unwrap();

        let result = script.run_once("foo", "bar").unwrap();
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
        "#.as_bytes()).unwrap();

        let result = script.run_once("foo", "bar").unwrap();
        assert!(result);
    }

    #[test]
    fn verify_basic_auth_correct() {
        let script = Script::load_from(r#"
        descr = "basic auth httpbin.org"

        function verify(user, password)
            return http_basic_auth("https://httpbin.org/basic-auth/foo/buzz", user, password)
        end
        "#.as_bytes()).unwrap();

        let result = script.run_once("foo", "buzz").unwrap();
        assert!(result);
    }

    #[test]
    fn verify_basic_auth_incorrect() {
        let script = Script::load_from(r#"
        descr = "basic auth httpbin.org"

        function verify(user, password)
            return http_basic_auth("https://httpbin.org/basic-auth/foo/buzz", user, password)
        end
        "#.as_bytes()).unwrap();

        let result = script.run_once("invalid", "wrong").unwrap();
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
        "#.as_bytes()).unwrap();

        let result = script.run_once("x", "x").unwrap();
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
        "#.as_bytes()).unwrap();

        let result = script.run_once("x", "x").unwrap();
        assert!(result);
    }
}
