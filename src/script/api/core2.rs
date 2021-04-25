use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
};

apigen::declare_ns!(core {
    struct {
        api: Api,
    }

    #[property]
    pub fn current(&self) -> CurrentObjects {
        CurrentObjects::new(self.api.clone())
    }
});

apigen::declare_ns!(CurrentObjects {
    struct {
        api: Api,
    }

    #[property(api)]
    pub fn current_buffer(&self) -> Id {
        context.state().current_buffer().id()
    }

    #[rpc]
    pub fn buffer(&self) -> Result<BufferObject> {
        let id = self.api.current_buffer()?;
        BufferObject::new(self.api, id)
    }

    #[property(v2)]
    pub fn buffer(&self) -> Result<BufferObject> {
        let id = rpc(context) -> Result<Id> {
            Ok(context.state().current_buffer().id())
        }?;
        BufferObject::new(self.api, id)
    }

    #[property(v3)]
    pub fn buffer(&self) -> Result<BufferObject> {
        rpc fn(context) -> Result<Id> {
            Ok(context.state().current_buffer().id())
        }
        let id = rpc()?;
        BufferObject::new(self.api, id)
    }
});

apigen::declare_ns!(BufferObject {
    struct {
        api: Api,
        id: Id,
    }

    #[property]
    pub fn name(&self, context: &mut CommandHandlerContext) -> Option<String> {
        if let Some(buf) = context.state().buffers.by_id(id) {
            Some(format!("{:?}", buf.source()))
        } else {
            None
        }
    }

    #[property(v2)]
    pub fn name(&self) -> Result<Option<String>> {
        rpc(context, id: Id = self.id) -> Result<Option<String>> {
            Ok(context
               .state()
               .buffers.by_id(id)
               .and_then(|buf| Some(format!("{:?}", buf.source()))))
        }
    }

    #[property(v3)]
    pub fn name(&self) -> Result<Option<String>> {
        rpc fn(context, id: Id) -> Result<Option<String>> {
            Ok(context
                .state()
                .buffers.by_id(id)
                .and_then(|buf| Some(format!("{:?}", buf.source()))))
        }
        rpc(self.id)
    }

    #[property(vDirect)]
    pub fn name(&self, context: &mut CommandHandlerContext, id: Id = self.id) -> Result<Option<String>> {
        Ok(context
           .state()
           .buffers.by_id(id)
           .and_then(|buf| Some(format!("{:?}", buf.source()))))
    }
});

// type Api = usize;
//
// #[apigen::ns]
// pub struct IaidoCore {
//     api: Api,
// }
//
// #[apigen::ns_impl]
// impl IaidoCore {
//     #[property]
//     pub fn current(&self) -> CurrentObjects {
//         CurrentObjects::new(self.api.clone())
//     }
// }
//
// #[apigen::ns]
// pub struct CurrentObjects {
//     api: Api,
// }
//
// #[apigen::ns_impl]
// impl CurrentObjects {
//     pub fn new(api: Api) -> Self {
//         Self { api }
//     }
//
//     #[rpc]
//     pub fn buffer_id(context: &mut CommandHandlerContext) -> Id {
//         context.state().current_buffer().id()
//     }
//
//     pub fn buffer_id(&self) -> Id {
//         self.api
//             .perform(CurrentObjectsApi, CurrentObjectsApiRequest::buffer_id)
//     }
//
//     #[property]
//     pub fn buffer(&self) -> BufferApiObject {
//         // NOTE: buffer_id should be turned into a method that
//         // delegates to an RPC call
//         BufferApiObject::new(self.api, self.buffer_id())
//     }
// }
//
// #[apigen::ns]
// pub struct BufferApiObject {
//     api: Api,
//     pub id: Id,
// }
//
// #[apigen::ns_impl]
// impl BufferApiObject {
//     pub fn new(api: Api, id: Id) -> Self {
//         Self { api, id }
//     }
//
//     #[property]
//     pub fn name(&self) -> CurrentObjects {}
// }
