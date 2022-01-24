use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use crate::{
    editing::{buffer::BufHidden, gutter::Gutter, source::BufferSource},
    input::{
        keys::KeysParsable,
        maps::{KeyHandlerContext, KeyResult},
        KeymapContext, RemapMode, Remappable,
    },
};
use crate::{
    editing::{text::EditableLine, Id},
    input::{history::History, Key, KeyCode},
};

use super::VimKeymap;

#[derive(Clone, Copy, Debug)]
pub enum CmdlineSink {
    SubmitPrompt(&'static str),
    ConnectionBuffer(Id),
}

fn cmdline_to_prompt(
    mut ctx: KeyHandlerContext<VimKeymap>,
    sink: CmdlineSink,
) -> KeyResult<(bool, KeyHandlerContext<VimKeymap>)> {
    let cmd = if let Some(cmd_spans) = ctx
        .state()
        .current_buffer()
        .checked_get(ctx.state().current_window().cursor.line)
    {
        cmd_spans.to_string()
    } else {
        "".to_string()
    };

    // Release the buffer
    let buffer_id = ctx.state().current_buffer().id();
    ctx.state_mut().delete_buffer(buffer_id);

    let can_submit = match sink {
        CmdlineSink::SubmitPrompt(prompt_key) => {
            // Is this *too* hacky? Just feed each char as a key:
            // Perhaps we should match on prompt_key and invoke eg `handle_command`,
            // `handle_forward_search`, etc. directly...
            ctx = ctx.feed_keys_noremap(prompt_key.into_keys())?;

            let cmd_as_keys: Vec<Key> =
                cmd.chars().map(|ch| Key::from(KeyCode::Char(ch))).collect();
            ctx = ctx.feed_keys_noremap(cmd_as_keys)?;
            true
        }

        CmdlineSink::ConnectionBuffer(conn_buffer_id) => {
            if let Some(input_buffer_id) = ctx
                .state_mut()
                .connections
                .as_mut()
                .and_then(|conns| conns.by_buffer_id(conn_buffer_id))
                .map(|conn| conn.id)
                .and_then(|conn_id| ctx.state().conn_input_buffer_id(conn_id))
            {
                let found_window = ctx
                    .state_mut()
                    .current_tab_mut()
                    .set_focus_to_buffer(input_buffer_id)
                    .is_some();
                match ctx.state_mut().winsbuf_by_id(input_buffer_id) {
                    Some(mut winsbuf) if !cmd.is_empty() => {
                        winsbuf.clear();
                        winsbuf.append(cmd.into());
                        found_window
                    }

                    _ => {
                        // No winsbuf or nothing selected; it's unlikely that there's no winsbuf,
                        // but perhaps we got disconnected or otherwise tore down the input UI
                        // while the cmdline window was open?
                        // If we selected nothing, we *probably* don't want to override whatever
                        // might have been in the winsbuf before, and we *definitely* don't want to
                        // send it
                        false
                    }
                }
            } else {
                // Couldn't find the input buffer
                false
            }
        }
    };

    Ok((can_submit, ctx))
}

fn cancel_cmdline(ctx: KeyHandlerContext<VimKeymap>, sink: CmdlineSink) -> KeyResult {
    cmdline_to_prompt(ctx, sink)?;
    Ok(())
}

fn submit_cmdline(ctx: KeyHandlerContext<VimKeymap>, sink: CmdlineSink) -> KeyResult {
    let (can_submit, ctx) = cmdline_to_prompt(ctx, sink)?;
    if can_submit {
        match sink {
            CmdlineSink::SubmitPrompt(_) => {
                ctx.feed_keys_noremap("<cr>".into_keys())?;
            }
            CmdlineSink::ConnectionBuffer(_) => {
                ctx.feed_keys_noremap("a<cr><esc>".into_keys())?;
            }
        }
    }
    Ok(())
}

pub fn open_from_history(
    ctx: &mut KeyHandlerContext<VimKeymap>,
    history: &History<String>,
    history_key: String,
    sink: CmdlineSink,
) -> KeyResult<()> {
    ctx.state_mut().clear_echo();

    let win_id = ctx.state_mut().current_tab_mut().split_bottom();
    let buffer = ctx.state_mut().buffers.create_mut();
    let buf_id = buffer.id();
    buffer.set_source(BufferSource::Cmdline);
    buffer.config_mut().bufhidden = BufHidden::Delete;

    let mut count = 0;
    for entry in history.iter().rev() {
        buffer.append_line(entry.to_string());
        count += 1;
    }

    ctx.state_mut().current_tab_mut().set_focus(win_id);
    ctx.state_mut().set_current_window_buffer(buf_id);

    // Bind <cr> to submit the input
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimNormal,
        "<cr>".into_keys(),
        Box::new(move |ctx| submit_cmdline(ctx, sink)),
    );
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimInsert,
        "<cr>".into_keys(),
        Box::new(move |ctx| submit_cmdline(ctx, sink)),
    );

    // Bind <ctrl-c> to cancel the mode
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimNormal,
        "<ctrl-c>".into_keys(),
        Box::new(move |ctx| cancel_cmdline(ctx, sink)),
    );
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimInsert,
        "<ctrl-c>".into_keys(),
        Box::new(move |ctx| cancel_cmdline(ctx, sink)),
    );

    let win = ctx.state_mut().current_tab_mut().by_id_mut(win_id).unwrap();

    // TODO Resize to cmdwinheight

    let non_line_prefix = vec![Span::styled("~", Style::default().fg(Color::DarkGray))];

    let gutter_prefix = vec![Span::styled(
        history_key,
        Style::default().fg(Color::DarkGray),
    )];

    win.gutter = Some(Gutter {
        width: 1,
        get_content: Box::new(move |line| {
            Spans(match line {
                Some(_) => gutter_prefix.clone(),
                None => non_line_prefix.clone(),
            })
        }),
    });
    win.cursor = (count, 0).into();

    Ok(())
}

pub fn open(
    mut ctx: KeyHandlerContext<VimKeymap>,
    history_key: String,
    sink: CmdlineSink,
) -> KeyResult<()> {
    let history = ctx.keymap.histories.take(&history_key);

    open_from_history(&mut ctx, &history, history_key.clone(), sink)?;

    ctx.keymap
        .histories
        .replace(history_key.to_string(), history);

    Ok(())
}
