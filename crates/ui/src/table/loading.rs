use crate::{h_flex, skeleton::Skeleton, v_flex, ActiveTheme, Size};
use gpui::{prelude::FluentBuilder as _, IntoElement, ParentElement as _, RenderOnce, Styled};

#[derive(IntoElement)]
pub struct Loading {
    size: Size,
}

impl Loading {
    pub fn new() -> Self {
        Self { size: Size::Medium }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

#[derive(IntoElement)]
struct LoadingRow {
    header: bool,
    size: Size,
}

impl LoadingRow {
    pub fn header() -> Self {
        Self {
            header: true,
            size: Size::Medium,
        }
    }

    pub fn row() -> Self {
        Self {
            header: false,
            size: Size::Medium,
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for LoadingRow {
    fn render(self, _: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let paddings = self.size.table_cell_padding();
        let height = self.size.table_row_height() * 0.5;

        h_flex()
            .gap_3()
            .h(self.size.table_row_height())
            .overflow_hidden()
            .pt(paddings.top)
            .pb(paddings.bottom)
            .pl(paddings.left)
            .pr(paddings.right)
            .items_center()
            .justify_between()
            .overflow_hidden()
            .when(self.header, |this| this.bg(cx.theme().table_head))
            .when(!self.header, |this| {
                this.border_t_1().border_color(cx.theme().table_row_border)
            })
            .child(
                h_flex()
                    .gap_3()
                    .flex_1()
                    .child(Skeleton::new().secondary(self.header).h(height).w_24())
                    .child(Skeleton::new().secondary(self.header).h(height).w_48())
                    .child(Skeleton::new().secondary(self.header).h(height).w_16()),
            )
            .child(Skeleton::new().secondary(self.header).h(height).w_24())
    }
}

impl RenderOnce for Loading {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        v_flex()
            .gap_0()
            .child(LoadingRow::header().size(self.size))
            .child(LoadingRow::row().size(self.size))
            .child(LoadingRow::row().size(self.size))
            .child(LoadingRow::row().size(self.size))
            .child(LoadingRow::row().size(self.size))
    }
}
