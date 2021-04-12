use crate::input::KeyError;

pub mod core;
mod manager;

pub use manager::{ApiManager, ApiManagerDelegate};

use self::core::ScriptingFnRef;

pub enum ApiRequest {
    Echo(String),
    SetKeymapFn(String, String, ScriptingFnRef),
}

pub type ApiResult = Result<(), KeyError>;

pub trait ApiDelegate {
    fn perform(&self, request: ApiRequest) -> ApiResult;
}
