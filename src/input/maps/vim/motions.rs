use crate::input::maps::vim::VimKeymap;
use crate::input::{Key, KeyCode, KeySource, KeymapContext};
use crate::vim_tree;
use crate::{
    editing::motion::char::CharMotion,
    editing::motion::end::EndOfWordMotion,
    editing::motion::find::FindMotion,
    editing::motion::linewise::{
        DownLineMotion, ToFirstLineMotion, ToLastLineMotion, ToLineEndMotion, ToLineStartMotion,
        UpLineMotion,
    },
    editing::motion::word::{is_big_word_boundary, is_small_word_boundary, WordMotion},
};

use super::tree::KeyTreeNode;

/// Motions shared across all types of vim navigation
pub fn vim_standard_motions() -> KeyTreeNode {
    vim_tree! {
        "b" => motion { WordMotion::backward_until(is_small_word_boundary) },
        "B" => motion { WordMotion::backward_until(is_big_word_boundary) },
        "w" => motion { WordMotion::forward_until(is_small_word_boundary) },
        "W" => motion { WordMotion::forward_until(is_big_word_boundary) },

        "ge" => motion { EndOfWordMotion::backward_until(is_small_word_boundary) },
        "gE" => motion { EndOfWordMotion::backward_until(is_big_word_boundary) },
        "e" => motion { EndOfWordMotion::forward_until(is_small_word_boundary) },
        "E" => motion { EndOfWordMotion::forward_until(is_big_word_boundary) },

        "h" => motion { CharMotion::Backward(1) },
        "l" => motion { CharMotion::Forward(1) },

        "0" => motion { ToLineStartMotion },
        "$" => motion { ToLineEndMotion },

        "f" => motion |ctx| {
            match ctx.next_key()? {
                Some(Key { code: KeyCode::Char(ch), .. }) => FindMotion::forward_to(ch),
                _ =>{  return Ok(()); }
            }
        },
        "F" => motion |ctx| {
            match ctx.next_key()? {
                Some(Key { code: KeyCode::Char(ch), .. }) => FindMotion::backward_to(ch),
                _ =>{  return Ok(()); }
            }
        },
    }
}

/// Motions that should only be used for linewise vim (IE: not input mode)
pub fn vim_linewise_motions() -> KeyTreeNode {
    vim_tree! {
        "j" => motion { DownLineMotion },
        "k" => motion { UpLineMotion },

        "gg" => motion { ToFirstLineMotion },
        "G" => motion { ToLastLineMotion },
    }
}
