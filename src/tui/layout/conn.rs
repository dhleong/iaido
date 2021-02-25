use std::cmp::{max, min};

use crate::{
    editing::{layout::conn::ConnLayout, Resizable, Size},
    tui::{measure::Measurable, LayoutContext, RenderContext, Renderable},
};

const MIN_OUTPUT_HEIGHT: u16 = 3;
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
        let input_height = min(
            preferred_height,
            min(available_input_height, MAX_INPUT_HEIGHT),
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
    use crate::tui::rendering::display::tests::TestableDisplay;
    use crate::tui::tabpage::tests::tabpage;
    use crate::tui::Size;
    use indoc::indoc;

    #[test]
    fn resize_input_to_match_content() {
        let mut tabpage = tabpage(indoc! {"
            Take my love
        "});

        let conn = tabpage
            .tab
            .new_connection(&mut tabpage.buffers, tabpage.tab.current_window().buffer);

        let input_bufid = conn.input.buffer;

        tabpage
            .tab
            .replace_window(tabpage.tab.current_window().id, Box::new(conn));

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
}
