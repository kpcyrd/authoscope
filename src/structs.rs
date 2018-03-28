use hlua::{AnyHashableLuaValue, AnyLuaValue};

use std::collections;
use std::collections::HashMap;


#[derive(Debug)]
pub struct LuaMap(HashMap<AnyHashableLuaValue, AnyLuaValue>);

impl LuaMap {
    pub fn new() -> LuaMap {
        LuaMap(HashMap::new())
    }

    pub fn insert<K: Into<String>, V: Into<AnyLuaValue>>(&mut self, k: K, v: V) {
        self.0.insert(AnyHashableLuaValue::LuaString(k.into()), v.into());
    }

    pub fn insert_str<K: Into<String>, V: Into<String>>(&mut self, k: K, v: V) {
        self.0.insert(AnyHashableLuaValue::LuaString(k.into()), AnyLuaValue::LuaString(v.into()));
    }

    pub fn insert_num<K: Into<String>>(&mut self, k: K, v: f64) {
        self.0.insert(AnyHashableLuaValue::LuaString(k.into()), AnyLuaValue::LuaNumber(v));
    }

    // TODO: use trait instead
    pub fn into_iter(self) -> collections::hash_map::IntoIter<AnyHashableLuaValue, AnyLuaValue> {
        self.0.into_iter()
    }
}

impl From<HashMap<AnyHashableLuaValue, AnyLuaValue>> for LuaMap {
    fn from(x: HashMap<AnyHashableLuaValue, AnyLuaValue>) -> LuaMap {
        LuaMap(x)
    }
}

impl From<Vec<(AnyLuaValue, AnyLuaValue)>> for LuaMap {
    fn from(x: Vec<(AnyLuaValue, AnyLuaValue)>) -> LuaMap {
        let mut map = LuaMap::new();

        for (k, v) in x {
            match k {
                AnyLuaValue::LuaString(k) => map.insert(k, v),
                _ => (), // TODO: handle unknown types
            }
        }

        map
    }
}

impl Into<HashMap<AnyHashableLuaValue, AnyLuaValue>> for LuaMap {
    fn into(self: LuaMap) -> HashMap<AnyHashableLuaValue, AnyLuaValue> {
        self.0
    }
}

impl Into<AnyLuaValue> for LuaMap {
    fn into(self: LuaMap) -> AnyLuaValue {
        AnyLuaValue::LuaArray(
            self.into_iter()
                .filter_map(|(k, v)| {
                    match k {
                        AnyHashableLuaValue::LuaString(x) => Some((AnyLuaValue::LuaString(x), v)),
                        _ => None, // TODO: unknown types are discarded
                    }
                })
                .collect()
        )
    }
}
