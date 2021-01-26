use crate::input::KeymapContext;
use crate::vim_tree;
use crate::{
    editing::motion::char::CharMotion,
    editing::motion::linewise::{DownLineMotion, ToLineEndMotion, ToLineStartMotion, UpLineMotion},
    editing::motion::word::{is_big_word_boundary, is_small_word_boundary, WordMotion},
    key_handler,
};

use super::tree::KeyTreeNode;
use super::VimKeymapState;

/// Motions shared across all types of vim navigation
pub fn vim_standard_motions<'a>() -> KeyTreeNode<'a> {
    vim_tree! {
        "b" => motion { WordMotion::backward_until(is_small_word_boundary) },
        "B" => motion { WordMotion::backward_until(is_big_word_boundary) },
        "w" => motion { WordMotion::forward_until(is_small_word_boundary) },
        "W" => motion { WordMotion::forward_until(is_big_word_boundary) },

        "h" => motion { CharMotion::Backward(1) },
        "l" => motion { CharMotion::Forward(1) },

        "0" => motion { ToLineStartMotion },
        "$" => motion { ToLineEndMotion },
    }
}

/// Motions that should only be used for linewise vim (IE: not input mode)
pub fn vim_linewise_motions<'a>() -> KeyTreeNode<'a> {
    vim_tree! {
        "j" => motion { DownLineMotion },
        "k" => motion { UpLineMotion },
    }
}
