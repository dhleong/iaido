use crate::input::maps::KeyResult;

use super::{ApiDelegate, ApiRequest};

pub struct IaidoApi<A: ApiDelegate> {
    api: A,
}

impl<A: ApiDelegate> IaidoApi<A> {
    pub fn new(api: A) -> Self {
        Self { api }
    }

    pub fn echo(&self, message: String) -> KeyResult {
        self.api.perform(ApiRequest::Echo(message))
    }
}
