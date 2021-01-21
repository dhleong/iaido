use async_trait::async_trait;
use futures::future::LocalBoxFuture;

use super::{KeySource, KeymapContext};

pub mod vim;

pub type AsyncKeymapContext = dyn KeymapContext + Send + Sync;
pub struct KeyHandlerContext<'a, T: Send + Sync> {
    context: &'a mut Box<AsyncKeymapContext>,
    state: &'a mut T,
}

impl<'a, T: Send + Sync> KeymapContext for KeyHandlerContext<'a, T> {
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

pub type BoxedResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type KeyResult = BoxedResult<()>;
pub type KeyHandler<'a, T> =
    dyn Fn(&'a mut KeyHandlerContext<'a, T>) -> LocalBoxFuture<'a, KeyResult> + Send + Sync;
