use std::ops::Range;

use gpui::{
    div, App, Context, Div, InteractiveElement as _, IntoElement, ParentElement as _, Stateful,
    Styled as _, Window,
};

use crate::{
    h_flex,
    popup_menu::PopupMenu,
    table::{loading::Loading, Column, ColumnSort, Table},
    ActiveTheme as _, Icon, IconName, Size,
};

#[allow(unused)]
pub trait TableDelegate: Sized + 'static {
    /// Return the number of columns in the table.
    fn columns_count(&self, cx: &App) -> usize;
    /// Return the number of rows in the table.
    fn rows_count(&self, cx: &App) -> usize;

    /// Returns the table column at the given index.
    ///
    /// This only call on Table prepare or refresh.
    fn column(&self, col_ix: usize, cx: &App) -> &Column;

    /// Perform sort on the column at the given index.
    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) {
    }

    /// Render the header cell at the given column index, default to the column name.
    fn render_th(
        &self,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .child(self.column(col_ix, cx).name.clone())
    }

    /// Render the row at the given row and column.
    fn render_tr(
        &self,
        row_ix: usize,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> Stateful<Div> {
        h_flex().id(("row", row_ix))
    }

    /// Render the context menu for the row at the given row index.
    fn context_menu(&self, row_ix: usize, menu: PopupMenu, window: &Window, cx: &App) -> PopupMenu {
        menu
    }

    /// Render cell at the given row and column.
    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement;

    /// Move the column at the given `col_ix` to insert before the column at the given `to_ix`.
    fn move_column(
        &mut self,
        col_ix: usize,
        to_ix: usize,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) {
    }

    /// Return a Element to show when table is empty.
    fn render_empty(&self, window: &mut Window, cx: &mut Context<Table<Self>>) -> impl IntoElement {
        h_flex()
            .size_full()
            .justify_center()
            .text_color(cx.theme().muted_foreground.opacity(0.6))
            .child(Icon::new(IconName::Inbox).size_12())
            .into_any_element()
    }

    /// Return true to show the loading view.
    fn loading(&self, cx: &App) -> bool {
        false
    }

    /// Return a Element to show when table is loading, default is built-in Skeleton loading view.
    ///
    /// The size is the size of the Table.
    fn render_loading(
        &self,
        size: Size,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        Loading::new().size(size)
    }

    /// Return true to enable load more data when scrolling to the bottom.
    ///
    /// Default: true
    fn is_eof(&self, cx: &App) -> bool {
        true
    }

    /// Returns a threshold value (n rows), of course, when scrolling to the bottom,
    /// the remaining number of rows triggers `load_more`.
    /// This should smaller than the total number of first load rows.
    ///
    /// Default: 20 rows
    fn load_more_threshold(&self) -> usize {
        20
    }

    /// Load more data when the table is scrolled to the bottom.
    ///
    /// This will performed in a background task.
    ///
    /// This is always called when the table is near the bottom,
    /// so you must check if there is more data to load or lock the loading state.
    fn load_more(&mut self, window: &mut Window, cx: &mut Context<Table<Self>>) {}

    /// Render the last empty column, default to empty.
    fn render_last_empty_col(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        h_flex().w_3().h_full().flex_shrink_0()
    }

    /// Called when the visible range of the rows changed.
    ///
    /// NOTE: Make sure this method is fast, because it will be called frequently.
    ///
    /// This can used to handle some data update, to only update the visible rows.
    /// Please ensure that the data is updated in the background task.
    fn visible_rows_changed(
        &mut self,
        visible_range: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) {
    }

    /// Called when the visible range of the columns changed.
    ///
    /// NOTE: Make sure this method is fast, because it will be called frequently.
    ///
    /// This can used to handle some data update, to only update the visible rows.
    /// Please ensure that the data is updated in the background task.
    fn visible_columns_changed(
        &mut self,
        visible_range: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) {
    }
}
