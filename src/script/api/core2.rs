use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
};

type Api = usize;

#[apigen::ns]
pub struct IaidoCore {
    api: Api,
}

#[apigen::ns_impl]
impl IaidoCore {
    #[property]
    pub fn current(&self) -> CurrentObjects {
        CurrentObjects::new(self.api.clone())
    }
}

#[apigen::ns]
pub struct CurrentObjects {
    api: Api,
}

#[apigen::ns_impl]
impl CurrentObjects {
    pub fn new(api: Api) -> Self {
        Self { api }
    }

    #[rpc]
    pub fn buffer_id(context: &mut CommandHandlerContext) -> Id {
        context.state().current_buffer().id()
    }

    // expands to:
    // pub fn buffer_id(&self) -> Id {
    //   match self.api.perform(CurrentObjectsApi, CurrentObjectsApiRequest::buffer_id) {
    //      Ok(CurrentObjectsApiResponse::buffer_id(id)) => id,
    //      Ok(unexpected) => panic!("Unexpected response: {:?}", unexpected),
    //      Err(e) => std::panic::panic_any(e),
    //   }
    // }
    // ...
    // enum CurrentObjectsApiRequest {
    //   buffer_id,
    // }
    // enum CurrentObjectsApiResponse {
    //   buffer_id(Id),
    // }
    // struct CurrentObjectsApi;
    // impl ApiHandler<CurrentObjectsApiRequest> for CurrentObjectsApi {
    //  fn handle(
    //    &self,
    //    context: &mut CommandHandlerContext,
    //    p: CurrentObjectsApiRequest
    //  ) -> ApiResult {
    //    match p {
    //      CurrentObjectsApiRequest::buffer_id => {
    //        Ok(
    //         CurrentObjectsApiResponse::buffer_id(
    //           CurrentObjects::buffer_id(context),
    //         )
    //        )
    //      }
    //    }
    //  }
    // }

    #[property]
    pub fn buffer(&self) -> BufferApiObject {
        // NOTE: buffer_id should be turned into a method that
        // delegates to an RPC call
        BufferApiObject::new(self.api, self.buffer_id())
    }
}

#[apigen::ns]
pub struct BufferApiObject {
    api: Api,
    pub id: Id,
}

#[apigen::ns_impl]
impl BufferApiObject {
    pub fn new(api: Api, id: Id) -> Self {
        Self { api, id }
    }

    #[property]
    #[rpc(self.id)]
    pub fn name(context: &mut CommandHandlerContext, id: Id) -> Option<String> {
        if let Some(buf) = context.state().buffers.by_id(id) {
            Some(format!("{:?}", buf.source()))
        } else {
            None
        }
    }

    // expands to:
    // pub fn name(&self) -> KeyResult<Option<String>> {
    //     match self.api
    //         .perform(BufferApiObjectApi, BufferApiObjectApiRequeset::name(self.id)) {
    //       Ok(BufferApiObjectApiResponse::name(response)) => Ok(response),
    //       Ok(unexpected) => panic!("Unexpected response: {:?}"),
    //       Err(e) => Err(e)
    //     }
    // }
    // ...
    // enum BufferApiObjectApiRequest {
    //   name(Id),
    // }
    // enum BufferApiObjectApiResponse {
    //   name(Option<String>),
    // }
    // struct BufferApiObjectApi;
    // impl ApiHandler<BufferApiObjectApiRequest> for BufferApiObjectApi {
    //  fn handle(
    //    &self,
    //    context: &mut CommandHandlerContext,
    //    p: BufferApiObjectApiRequest
    //  ) -> ApiResult {
    //    match p {
    //      BufferApiObjectApiRequest::name(id) => {
    //        let result = BufferApiObject::name(context, id);
    //        Ok(result)
    //      }
    //    }
    //  }
    // }
}
