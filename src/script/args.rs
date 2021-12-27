use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum FnArgs {
    None,
    Bool(bool),
    String(String),
    Map(HashMap<String, FnArgs>),
}

impl Into<FnArgs> for HashMap<String, String> {
    fn into(self) -> FnArgs {
        let mut m: HashMap<String, FnArgs> = Default::default();

        for (k, v) in self {
            m.insert(k, FnArgs::String(v));
        }

        FnArgs::Map(m)
    }
}

pub trait FnReturnable {
    fn is_string(&self) -> bool;
    fn is_truthy(&self) -> bool;

    fn to_string(&self) -> Option<String>;
}

pub type FnReturnValue = Option<Box<dyn FnReturnable>>;
