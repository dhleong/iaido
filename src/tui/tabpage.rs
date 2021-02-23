use super::Renderable;
use crate::editing::tabpage::Tabpage;

impl Renderable for Tabpage {
    fn layout(&mut self, ctx: &super::LayoutContext) {
        self.layout.layout(ctx);
    }

    fn render(&self, app: &mut crate::tui::RenderContext) {
        self.layout.render(app);
    }
}

#[cfg(test)]
mod tests {
    use tui::layout::Rect;

    use crate::{
        app,
        editing::{buffers::Buffers, motion::tests::window, text::TextLines, Resizable, Size},
        tui::{Display, RenderContext},
    };

    use super::*;

    use crate::tui::rendering::display::tests::TestableDisplay;
    use indoc::indoc;

    pub struct TestableTabpage {
        pub tab: Tabpage,
        pub buffers: Buffers,
        pub size: Size,
    }

    pub fn tabpage(s: &'static str) -> TestableTabpage {
        let window = window(s);
        let Size { w, h } = window.window.size;
        let mut page = tabpage_of_size(w, h);

        let id = page.buffers.create().id();
        page.buffers
            .by_id_mut(id)
            .unwrap()
            .append(TextLines::raw(s.replace("|", "")));
        page.tab.current_window_mut().cursor = window.window.cursor;
        page.tab.current_window_mut().buffer = id;

        page
    }

    pub fn tabpage_of_size(w: u16, h: u16) -> TestableTabpage {
        let mut buffers = Buffers::new();
        let size = Size { w, h };
        TestableTabpage {
            tab: Tabpage::new(0, &mut buffers, size),
            buffers,
            size,
        }
    }

    impl TestableTabpage {
        pub fn render(mut self) -> Display {
            let mut state = app::State::default();
            state.buffers = self.buffers;

            let area: Rect = self.size.into();
            let mut display = Display::new(area.into());
            let mut context = RenderContext {
                app: &state,
                display: &mut display,
                area,
                buffer_override: None,
            };
            self.tab.resize(self.size);
            self.tab.render(&mut context);

            display
        }
    }

    #[test]
    fn vertical_test() {
        let mut tabpage = tabpage(indoc! {"
            Take my love
        "});
        tabpage.tab.hsplit();
        tabpage.tab.hsplit();
        tabpage.size = Size { w: 12, h: 5 };

        tabpage.render().assert_visual_equals(indoc! {"
            Take my love
            ────────────
            Take my love
            ────────────
            Take my love
        "});
    }

    #[test]
    fn horizontal_test() {
        let mut tabpage = tabpage(indoc! {"
            Take my love
        "});
        tabpage.tab.vsplit();
        tabpage.tab.vsplit();
        tabpage.size = Size { w: 14, h: 3 };

        tabpage.render().assert_visual_equals(indoc! {"
            Take│Take│Take
            my  │my  │my  
            love│love│love
        "});
    }

    #[test]
    fn split_test() {
        let mut tabpage = tabpage(indoc! {"
            Take my love
        "});
        tabpage.tab.hsplit();
        tabpage.tab.vsplit();
        tabpage.size = Size { w: 14, h: 3 };

        tabpage.render().assert_visual_equals(indoc! {"
            Take my love
            ──────────────
            love   │love
        "});
    }
}
