use router::Params as RouteParams;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Params {
    i: HashMap<String, String>,
}

impl Params {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.i.get(key).map(|m| m.as_str())
    }
}

impl<'a> RouteParams<'a> for Params {
    fn set(&mut self, key: std::borrow::Cow<'a, str>, value: std::borrow::Cow<'a, str>) {
        self.i.insert(key.to_string(), value.to_string());
    }
}
