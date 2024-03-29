use crate::input::maps::vim::tree::KeyTreeNode;
use crate::input::maps::vim::VimKeymap;
use crate::input::maps::{KeyHandlerContext, KeyResult};
use crate::vim_tree;

fn push(ctx: KeyHandlerContext<VimKeymap>, digit: u32) -> KeyResult {
    ctx.keymap.push_count_digit(digit);
    Ok(())
}

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "1" => |?mut ctx| push(ctx, 1),
        "2" => |?mut ctx| push(ctx, 2),
        "3" => |?mut ctx| push(ctx, 3),
        "4" => |?mut ctx| push(ctx, 4),
        "5" => |?mut ctx| push(ctx, 5),
        "6" => |?mut ctx| push(ctx, 6),
        "7" => |?mut ctx| push(ctx, 7),
        "8" => |?mut ctx| push(ctx, 8),
        "9" => |?mut ctx| push(ctx, 9),

        // NOTE: We may never need this mapping, since the 0 motion
        // should usually override it... but it's not a terrible idea
        // to have as a backup
        "0" => |?mut ctx| {
            if ctx.keymap.count > 0 {
                push(ctx, 0)?;
            } else {
                ctx.keymap.reset();
            }
            Ok(())
        },
    }
}
