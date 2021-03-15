use crate::input::maps::KeyResult;

use super::{ApiDelegate, ApiRequest};

pub fn echo<A: ApiDelegate>(api: &A, message: String) -> KeyResult {
    api.perform(ApiRequest::Echo(message))
}
