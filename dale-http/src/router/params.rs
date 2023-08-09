use router::Params as RouteParams;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Params {
    i: Option<HashMap<String, String>>,
}

impl Params {
    pub const fn new() -> Params {
        Params { i: None }
    }
}

impl Params {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.i.as_ref().and_then(|i| i.get(key).map(|m| m.as_str()))
    }
}

impl<'a> RouteParams<'a> for Params {
    fn set(&mut self, key: std::borrow::Cow<'a, str>, value: std::borrow::Cow<'a, str>) {
        if self.i.is_none() {
            self.i = Some(HashMap::default());
        }
        self.i
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }
}
