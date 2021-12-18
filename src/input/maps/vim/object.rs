use crate::editing::motion::word::{is_big_word_boundary, is_small_word_boundary};
use crate::editing::object::word::WordObject;
use crate::input::maps::vim::VimKeymap;
use crate::vim_tree;

use super::tree::KeyTreeNode;

/// Text Objects shared across all types of vim navigation
pub fn vim_standard_objects() -> KeyTreeNode {
    vim_tree! {
        "iw" => motion { WordObject::inner(is_small_word_boundary) },
        "iW" => motion { WordObject::inner(is_big_word_boundary) },
        "aw" => motion { WordObject::outer(is_small_word_boundary) },
        "aW" => motion { WordObject::outer(is_big_word_boundary) },
    }
}
