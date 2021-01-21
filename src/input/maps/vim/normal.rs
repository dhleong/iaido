use crate::{
    input::{Key, KeyCode, KeyModifiers},
    key_handler,
};

use super::KeyTreeNode;

pub fn vim_normal_mode() -> KeyTreeNode {
    let mut root = KeyTreeNode::root();

    root.insert(
        &[Key::new(KeyCode::Enter, KeyModifiers::NONE)],
        key_handler!(|ctx| {
            ctx.state_mut().running = false;
            Ok(())
        }),
    );

    root
}
