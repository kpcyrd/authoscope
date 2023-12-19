use crate::hlua::{AnyHashableLuaValue, AnyLuaValue};

use std::collections::HashMap;
use crate::structs::LuaMap;


impl From<mysql::Params> for LuaMap {
    fn from(params: mysql::Params) -> LuaMap {
        match params {
            mysql::Params::Empty => LuaMap::new(),
            mysql::Params::Named(map) => {
                map.into_iter()
                    .flat_map(|(k, v)| {
                        String::from_utf8(k)
                            .map(|k| (
                                AnyHashableLuaValue::LuaString(k),
                                mysql_value_to_lua(v)
                            ))
                    })
                    .collect::<HashMap<AnyHashableLuaValue, AnyLuaValue>>()
                    .into()
            },
            mysql::Params::Positional(_) => unimplemented!(),
        }
    }
}

impl From<LuaMap> for mysql::Params {
    fn from(x: LuaMap) -> mysql::Params {
        if x.is_empty() {
            mysql::Params::Empty
        } else {
            let mut params = HashMap::default();

            for (k, v) in x {
                if let AnyHashableLuaValue::LuaString(k) = k {
                    params.insert(k.into_bytes(), lua_to_mysql_value(v));
                } else {
                    panic!("unsupported keys in map");
                }
            }

            mysql::Params::Named(params)
        }
    }
}

fn lua_to_mysql_value(value: AnyLuaValue) -> mysql::Value {
    match value {
        AnyLuaValue::LuaString(x) => mysql::Value::Bytes(x.into_bytes()),
        AnyLuaValue::LuaAnyString(x) => mysql::Value::Bytes(x.0),
        AnyLuaValue::LuaNumber(v) => if v % 1f64 == 0f64 {
            mysql::Value::Int(v as i64)
        } else {
            mysql::Value::Float(v as f32)
        },
        AnyLuaValue::LuaBoolean(x) => mysql::Value::Int(if x { 1 } else { 0 }),
        AnyLuaValue::LuaArray(_x) => unimplemented!(),
        AnyLuaValue::LuaNil => mysql::Value::NULL,
        AnyLuaValue::LuaOther => unimplemented!(),
    }
}

pub fn mysql_value_to_lua(value: mysql::Value) -> AnyLuaValue {
    use mysql::Value::*;
    match value {
        NULL => AnyLuaValue::LuaNil,
        Bytes(bytes) => AnyLuaValue::LuaString(String::from_utf8(bytes).unwrap()),
        Int(i) => AnyLuaValue::LuaNumber(i as f64),
        UInt(i) => AnyLuaValue::LuaNumber(i as f64),
        Float(i) => AnyLuaValue::LuaNumber(i as f64),
        Double(i) => AnyLuaValue::LuaNumber(i as f64),
        Date(_, _, _, _, _, _, _) => unimplemented!(),
        Time(_, _, _, _, _, _) => unimplemented!(),
    }
}
