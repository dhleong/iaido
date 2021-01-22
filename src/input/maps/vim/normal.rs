use crate::{
    input::{keys::KeysParsable, KeymapContext},
    key_handler,
};

use super::{KeyTreeNode, VimKeymapState};

macro_rules! vim_handler {
    (|$ctx_name:ident| $body:expr) => {{
        key_handler!(VimKeymapState | $ctx_name | $body)
    }};
}

macro_rules! vim_tree {
    ($root:ident => $keys:literal => |$ctx_name:ident| $body:expr) => {{
        $root.insert(&$keys.into_keys(), vim_handler!(|$ctx_name| $body));
    }};

    ($root:ident => $keys:literal => |$ctx_name:ident| $body:expr, $($keysn:literal => |$ctx_namen:ident| $bodyn:expr),+) => {{
        vim_tree! { $root => $keys => |$ctx_name| $body }
        vim_tree! { $root => $($keysn => |$ctx_namen| $bodyn),+ }
    }};
}

pub fn vim_normal_mode<'a>() -> KeyTreeNode<'a> {
    let mut root = KeyTreeNode::root();

    vim_tree! { root =>
        "<cr>" => |ctx| {
            ctx.state_mut().running = false;
            Ok(())
        },

        "d" => |ctx| {
            ctx.state.pending_motion_action_key = Some('d'.into());
            Ok(())
        }
    }

    root
}
