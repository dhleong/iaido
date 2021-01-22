use crate::{
    input::{Key, KeyCode, KeyModifiers, KeymapContext},
    key_handler,
};

use super::{KeyTreeNode, VimKeymapState};

macro_rules! vim_handler {
    (|$ctx_name:ident| $body:expr) => {{
        key_handler!(VimKeymapState | $ctx_name | $body)
    }};
}

pub fn vim_normal_mode<'a>() -> KeyTreeNode<'a> {
    let mut root = KeyTreeNode::root();

    root.insert(
        &[Key::new(KeyCode::Enter, KeyModifiers::NONE)],
        vim_handler!(|ctx| {
            ctx.state_mut().running = false;
            Ok(())
        }),
    );

    root.insert(
        &[Key::new(KeyCode::Char('d'), KeyModifiers::NONE)],
        vim_handler!(|ctx| {
            ctx.state.pending_motion_action_key =
                Some(Key::new(KeyCode::Char('d'), KeyModifiers::NONE));
            Ok(())
        }),
    );

    root
}
