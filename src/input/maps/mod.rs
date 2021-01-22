use async_trait::async_trait;
use futures::future::LocalBoxFuture;

use super::{KeySource, KeymapContext, DynamicAsyncError};

pub mod vim;

pub type AsyncKeymapContext = dyn KeymapContext + Send + Sync;
pub struct KeyHandlerContext<'a, T: Send + Sync> {
    context: Box<&'a mut AsyncKeymapContext>,
    state: &'a mut T,
}

impl<'a, T: Send + Sync> KeymapContext for KeyHandlerContext<'a, T> {
    fn state(&self) -> &crate::app::State {
        self.context.state()
    }

    fn state_mut(&mut self) -> &mut crate::app::State {
        self.context.state_mut()
    }
}

#[async_trait]
impl<'a, T: Send + Sync> KeySource for KeyHandlerContext<'a, T> {
    async fn next_key(&mut self) -> Result<Option<super::Key>, super::KeyError> {
        self.context.next_key().await
    }
}

pub type BoxedResult<T> = Result<T, DynamicAsyncError>;
pub type KeyResult = BoxedResult<()>;
pub type KeyHandler<'a, T> =
    dyn Fn(&'a mut KeyHandlerContext<'a, T>) -> LocalBoxFuture<KeyResult> + Send + Sync;
