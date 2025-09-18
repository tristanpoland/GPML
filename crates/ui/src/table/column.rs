use gpui::{
    div, prelude::FluentBuilder, px, Bounds, Context, Edges, Empty, EntityId, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, Styled as _, TextAlign, Window,
};

use crate::ActiveTheme as _;

/// Represents a column in a table, used for initializing table columns.
#[derive(Debug, Clone)]
pub struct Column {
    pub key: SharedString,
    pub name: SharedString,
    pub align: TextAlign,
    pub sort: Option<ColumnSort>,
    pub paddings: Option<Edges<Pixels>>,
    pub width: Pixels,
    pub fixed: Option<ColumnFixed>,
    pub resizable: bool,
    pub movable: bool,
    pub selectable: bool,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            key: SharedString::new(""),
            name: SharedString::new(""),
            align: TextAlign::Left,
            sort: None,
            paddings: None,
            width: px(100.),
            fixed: None,
            resizable: true,
            movable: true,
            selectable: true,
        }
    }
}

impl Column {
    /// Create a new column with the given key and name.
    pub fn new(key: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the column to be sortable with custom sort function, default is None (not sortable).
    ///
    /// See also [`Column::sortable`] to enable sorting with default.
    pub fn sort(mut self, sort: ColumnSort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Set whether the column is sortable, default is true.
    ///
    /// See also [`Column::sort`].
    pub fn sortable(mut self) -> Self {
        self.sort = Some(ColumnSort::Default);
        self
    }

    /// Set whether the column is sort with ascending order.
    pub fn ascending(mut self) -> Self {
        self.sort = Some(ColumnSort::Ascending);
        self
    }

    /// Set whether the column is sort with descending order.
    pub fn descending(mut self) -> Self {
        self.sort = Some(ColumnSort::Descending);
        self
    }

    /// Set the alignment of the column text, default is left.
    ///
    /// Only `text_left`, `text_right` is supported.
    pub fn text_right(mut self) -> Self {
        self.align = TextAlign::Right;
        self
    }

    /// Set the padding of the column, default is None.
    pub fn paddings(mut self, paddings: impl Into<Edges<Pixels>>) -> Self {
        self.paddings = Some(paddings.into());
        self
    }

    pub fn p_0(mut self) -> Self {
        self.paddings = Some(Edges::all(px(0.)));
        self
    }

    /// Set the width of the column, default is 100px.
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into();
        self
    }

    /// Set whether the column is fixed, default is false.
    pub fn fixed(mut self, fixed: impl Into<ColumnFixed>) -> Self {
        self.fixed = Some(fixed.into());
        self
    }

    /// Set whether the column is fixed on left side, default is false.
    pub fn fixed_left(mut self) -> Self {
        self.fixed = Some(ColumnFixed::Left);
        self
    }

    /// Set whether the column is resizable, default is true.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set whether the column is movable, default is true.
    pub fn movable(mut self, movable: bool) -> Self {
        self.movable = movable;
        self
    }

    /// Set whether the column is selectable, default is true.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

impl FluentBuilder for Column {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFixed {
    Left,
}

/// Used to sort the column runtime info in Table internal.
#[derive(Debug, Clone)]
pub(crate) struct ColGroup {
    pub(crate) column: Column,
    /// This is the runtime width of the column, we may update it when the column is resized.
    ///
    /// Including the width with next columns by col_span.
    pub(crate) width: Pixels,
    /// The bounds of the column in the table after it renders.
    pub(crate) bounds: Bounds<Pixels>,
}

impl ColGroup {
    pub(crate) fn is_resizable(&self) -> bool {
        self.column.resizable
    }
}

#[derive(Clone)]
pub(crate) struct DragColumn {
    pub(crate) entity_id: EntityId,
    pub(crate) name: SharedString,
    pub(crate) width: Pixels,
    pub(crate) col_ix: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ColumnSort {
    /// No sorting.
    #[default]
    Default,
    /// Sort in ascending order.
    Ascending,
    /// Sort in descending order.
    Descending,
}

impl Render for DragColumn {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_4()
            .py_1()
            .bg(cx.theme().table_head)
            .text_color(cx.theme().muted_foreground)
            .opacity(0.9)
            .border_1()
            .border_color(cx.theme().border)
            .shadow_md()
            .w(self.width)
            .min_w(px(100.))
            .max_w(px(450.))
            .child(self.name.clone())
    }
}

#[derive(Clone)]
pub(crate) struct ResizeColumn(pub (EntityId, usize));
impl Render for ResizeColumn {
    fn render(&mut self, _window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}
