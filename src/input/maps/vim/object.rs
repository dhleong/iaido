use crate::editing::motion::word::{is_big_word_boundary, is_small_word_boundary};
use crate::editing::object::pair::InnerPairObject;
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

        "i]" => motion { InnerPairObject::new('[', ']') },
        "i[" => motion { InnerPairObject::new('[', ']') },
        "a]" => motion { InnerPairObject::new('[', ']').into_outer() },
        "a[" => motion { InnerPairObject::new('[', ']').into_outer() },

        "i)" => motion { InnerPairObject::new('(', ')') },
        "i(" => motion { InnerPairObject::new('(', ')') },
        "ib" => motion { InnerPairObject::new('(', ')') },
        "a)" => motion { InnerPairObject::new('(', ')').into_outer() },
        "a(" => motion { InnerPairObject::new('(', ')').into_outer() },
        "ab" => motion { InnerPairObject::new('(', ')').into_outer() },

        "i>" => motion { InnerPairObject::new('<', '>') },
        "i<" => motion { InnerPairObject::new('<', '>') },
        "a>" => motion { InnerPairObject::new('<', '>').into_outer() },
        "a<" => motion { InnerPairObject::new('<', '>').into_outer() },

        "i}" => motion { InnerPairObject::new('{', '}') },
        "i{" => motion { InnerPairObject::new('{', '}') },
        "ib" => motion { InnerPairObject::new('{', '}') },
        "a}" => motion { InnerPairObject::new('{', '}').into_outer() },
        "a{" => motion { InnerPairObject::new('{', '}').into_outer() },
        "ab" => motion { InnerPairObject::new('{', '}').into_outer() },

        "i\"" => motion { InnerPairObject::within_line('"', '"') },
        "i'" => motion { InnerPairObject::within_line('\'', '\'') },
        "i`" => motion { InnerPairObject::within_line('`', '`') },
        "a\"" => motion { InnerPairObject::within_line('"', '"').into_outer() },
        "a'" => motion { InnerPairObject::within_line('\'', '\'').into_outer() },
        "a`" => motion { InnerPairObject::within_line('`', '`').into_outer() },
    }
}
