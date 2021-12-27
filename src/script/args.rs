use std::collections::HashMap;

pub enum FnArgs {
    None,
    Map(HashMap<String, String>),
}
