use crate::input::{maps::KeyResult, KeymapContext};

use super::CommandHandlerContext;

pub fn quit(mut context: CommandHandlerContext) -> KeyResult {
    context.state_mut().running = false;
    Ok(())
}
