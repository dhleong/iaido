use crate::input::{
    maps::{KeyHandlerContext, KeyResult},
    BoxableKeymap, KeymapContext,
};

pub fn verify_can_edit<T: BoxableKeymap>(context: &KeyHandlerContext<T>) -> KeyResult {
    if context.state().current_buffer().is_read_only() {
        return Err(crate::input::KeyError::ReadOnlyBuffer);
    }
    Ok(())
}
