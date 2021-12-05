use std::cmp::{max, min};

use crate::{
    editing::{layout::conn::ConnLayout, Resizable, Size},
    tui::{measure::Measurable, LayoutContext, RenderContext, Renderable},
};

const MIN_OUTPUT_HEIGHT: u16 = 3;
const MIN_INPUT_HEIGHT: u16 = 1; // input height should be *at least* 1
const MAX_INPUT_HEIGHT: u16 = 5;

impl Renderable for ConnLayout {
    fn layout(&mut self, ctx: &LayoutContext) {
        // stretch input to fit content; shrink output to fit input:
        let Size { w, .. } = self.output.size;
        let input_buffer = ctx.buffer(self.input.buffer).unwrap();
        let preferred_height = input_buffer.measure_height(self.input.size.w);
        let available_height = self.output.size.h + self.input.size.h;
        let available_input_height = max(
            available_height.checked_sub(MIN_OUTPUT_HEIGHT).unwrap_or(1),
            1,
        );
        let input_height = max(
            min(
                preferred_height,
                min(available_input_height, MAX_INPUT_HEIGHT),
            ),
            MIN_INPUT_HEIGHT,
        );

        if self.input.size.h != input_height {
            self.output.resize(Size {
                w,
                h: available_height - input_height,
            });
            self.input.resize(Size { w, h: input_height });
        }

        self.output.layout(ctx);
        self.input.layout(ctx);
    }

    fn render(&self, ctx: &mut RenderContext) {
        let mut layout_area = ctx.area.clone();
        layout_area.height = self.output.size.h;
        self.output.render(&mut ctx.with_area(layout_area));

        layout_area.y += self.output.size.h;
        layout_area.height = self.input.size.h;
        self.input.render(&mut ctx.with_area(layout_area));
    }
}

#[cfg(test)]
mod tests {
    use crate::tui::Size;
    use crate::tui::{rendering::display::tests::TestableDisplay, tabpage::tests::TestableTabpage};
    use crate::{editing::Id, tui::tabpage::tests::tabpage};
    use indoc::indoc;

    fn conn_tabpage(output_content: &'static str) -> (TestableTabpage, Id) {
        let mut tabpage = tabpage(output_content);

        let conn = tabpage
            .tab
            .new_connection(&mut tabpage.buffers, tabpage.tab.current_window().buffer);

        let input_bufid = conn.input.buffer;

        tabpage
            .tab
            .replace_window(tabpage.tab.current_window().id, Box::new(conn));

        (tabpage, input_bufid)
    }

    #[test]
    fn resize_input_to_match_content() {
        let (mut tabpage, input_bufid) = conn_tabpage(indoc! {"
            Take my love
        "});

        let buffer = tabpage.buffers.by_id_mut(input_bufid).unwrap();
        buffer.append("Take my land; take me where I cannot stand".into());

        tabpage.size = Size { w: 14, h: 6 };
        tabpage.render().assert_visual_equals(indoc! {"


            Take my love
            Take my land;
            take me where
            I cannot stand
        "});
    }

    #[test]
    fn split_on_input_splits_output() {
        let (mut tabpage, input_bufid) = conn_tabpage(indoc! {"
            Take my love
            Take my land
        "});

        let buffer = tabpage.buffers.by_id_mut(input_bufid).unwrap();
        buffer.append("shiny".into());
        assert_eq!(tabpage.tab.current_window().buffer, input_bufid);

        tabpage.tab.hsplit();

        tabpage.size = Size { w: 14, h: 6 };
        tabpage.render().assert_visual_equals(indoc! {"

            Take my love
            Take my land
            ──────────────
            Take my land
            shiny
        "});
    }
}
