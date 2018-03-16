use hlua;
use errors::Result;
use runtime;

use std::fs::File;
use std::io::prelude::*;


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

        let mut lua = Script::ctx();
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

    fn ctx<'a>() -> hlua::Lua<'a> {
        let mut lua = hlua::Lua::new();

        runtime::execve(&mut lua);
        runtime::mysql_connect(&mut lua);
        runtime::sleep(&mut lua);

        lua
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
        let mut lua = Script::ctx();
        lua.execute::<()>(&self.code)?;

        let verify: Result<_> = lua.get("verify").ok_or("verify undefined".into());
        let mut verify: hlua::LuaFunction<_> = verify?;

        let result: bool = match verify.call_with_args((user, password)) {
            Ok(res) => res,
            Err(err) => {
                let err = format!("execution failed: {:?}", err);
                return Err(err.into())
            },
        };
        Ok(result)
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
}
