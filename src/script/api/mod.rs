use crate::{editing::Id, input::maps::KeyResult};

pub mod core;
mod manager;
pub mod objects;

pub use manager::{ApiManager, ApiManagerDelegate};

use self::core::ScriptingFnRef;

pub enum IdType {
    Buffer,
    Connection,
    Window,
    Tab,
}

pub enum ApiRequest {
    CurrentId(IdType),
    Echo(String),
    SetKeymapFn(String, String, ScriptingFnRef),
}

pub enum ApiResponse {
    Id(Id),
}

pub type ApiResult = KeyResult<Option<ApiResponse>>;

pub trait ApiDelegate {
    fn perform(&self, request: ApiRequest) -> ApiResult;
}
