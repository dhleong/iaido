use std::collections::HashMap;

pub enum FnArgs {
    None,
    Map(HashMap<String, String>),
}

pub trait FnReturnable {
    fn is_string(&self) -> bool;

    fn to_string(&self) -> Option<String>;
}

pub type FnReturnValue = Option<Box<dyn FnReturnable>>;
