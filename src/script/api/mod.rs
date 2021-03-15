use crate::input::KeyError;

pub mod core;
mod manager;

pub use manager::{ApiManager, ApiManagerDelegate};

pub enum ApiRequest {
    Echo(String),
}

pub type ApiResult = Result<(), KeyError>;

pub trait ApiDelegate {
    fn perform(&self, request: ApiRequest) -> ApiResult;
}
