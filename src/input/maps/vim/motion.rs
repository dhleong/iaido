use crate::editing::motion::Motion;
use crate::input::maps::KeyHandlerContext;
use crate::input::maps::KeyResult;
use crate::input::KeymapContext;
use crate::VimKeymap;

pub fn apply_motion<T: Motion>(mut ctx: KeyHandlerContext<VimKeymap>, motion: T) -> KeyResult {
    let operator_fn = ctx.keymap.operator_fn.take();

    let result = if let Some(op) = operator_fn {
        // execute pending operator fn
        let range = motion.range(ctx.state());
        op(&mut ctx, range)
    } else {
        // no operator fn? just move the cursor
        motion.apply_cursor(ctx.state_mut());
        Ok(())
    };

    // Always reset state *after* executing the operator
    ctx.keymap.reset();

    result
}
