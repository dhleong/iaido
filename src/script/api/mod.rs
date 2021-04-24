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
    BufferName(Id),
    CurrentId(IdType),
    SetCurrentId(IdType, Id),
    Echo(String),
    SetKeymapFn(String, String, ScriptingFnRef),
    TypedClose(IdType, Id),
}

pub enum ApiResponse {
    Id(Id),
    String(String),
}

pub type ApiResult = KeyResult<Option<ApiResponse>>;

pub trait ApiDelegate {
    fn perform(&self, request: ApiRequest) -> ApiResult;
}
