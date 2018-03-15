use hlua;

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

pub fn sleep(lua: &mut hlua::Lua) {
    lua.set("sleep", hlua::function1(move |n: i32| -> i32 {
        thread::sleep(Duration::from_secs(n as u64));
        0
    }))
}
