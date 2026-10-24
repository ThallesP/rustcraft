//! net/minecraft/client/gui/layouts/GridLayout.java
//!
//! Children in rows and columns: each column as wide as its widest cell, each row as tall as its
//! tallest, a child spanning several split among them by `Divisor`, then each child placed in the
//! space its cells have by its settings. `addChild`'s overloads: `add_child` (a cell with the
//! defaults copied), `add_child_with` (the settings given), `add_child_spanning` (rows and
//! columns) and `add_child_spanning_with`.

use crate::com::mojang::math::divisor::Divisor;
use crate::net::minecraft::client::gui::layouts::abstract_layout::{
    AbstractChildWrapper, AbstractLayout,
};
use crate::net::minecraft::client::gui::layouts::layout::{self, Layout};
use crate::net::minecraft::client::gui::layouts::layout_element::{
    LayoutElement, LayoutElementRef,
};
use crate::net::minecraft::client::gui::layouts::layout_settings::LayoutSettings;
use crate::net::minecraft::util::mth::Mth;

pub struct GridLayout {
    base: AbstractLayout,
    children: Vec<ChildContainer>,
    default_cell_settings: LayoutSettings,
    row_spacing: i32,
    column_spacing: i32,
}

/// `GridLayout.ChildContainer`.
struct ChildContainer {
    wrapper: AbstractChildWrapper,
    row: i32,
    column: i32,
    occupied_rows: i32,
    occupied_columns: i32,
}

impl ChildContainer {
    fn get_last_occupied_row(&self) -> i32 {
        self.row + self.occupied_rows - 1
    }

    fn get_last_occupied_column(&self) -> i32 {
        self.column + self.occupied_columns - 1
    }
}

impl Default for GridLayout {
    fn default() -> GridLayout {
        GridLayout::new()
    }
}

impl GridLayout {
    pub fn new() -> GridLayout {
        GridLayout::at(0, 0)
    }

    pub fn at(x: i32, y: i32) -> GridLayout {
        GridLayout {
            base: AbstractLayout::new(x, y, 0, 0),
            children: Vec::new(),
            default_cell_settings: LayoutSettings::defaults(),
            row_spacing: 0,
            column_spacing: 0,
        }
    }

    pub fn add_child(
        &mut self,
        child: LayoutElementRef,
        row: i32,
        column: i32,
    ) -> LayoutElementRef {
        let settings = self.new_cell_settings();
        self.add_child_with(child, row, column, settings)
    }

    pub fn add_child_with(
        &mut self,
        child: LayoutElementRef,
        row: i32,
        column: i32,
        cell_settings: LayoutSettings,
    ) -> LayoutElementRef {
        self.add_child_spanning_with(child, row, column, 1, 1, cell_settings)
    }

    pub fn add_child_spanning(
        &mut self,
        child: LayoutElementRef,
        row: i32,
        column: i32,
        rows: i32,
        columns: i32,
    ) -> LayoutElementRef {
        let settings = self.new_cell_settings();
        self.add_child_spanning_with(child, row, column, rows, columns, settings)
    }

    /// Panics where Java throws `IllegalArgumentException` for no rows or columns.
    pub fn add_child_spanning_with(
        &mut self,
        child: LayoutElementRef,
        row: i32,
        column: i32,
        rows: i32,
        columns: i32,
        cell_settings: LayoutSettings,
    ) -> LayoutElementRef {
        if rows < 1 {
            panic!("Occupied rows must be at least 1");
        }
        if columns < 1 {
            panic!("Occupied columns must be at least 1");
        }
        self.children.push(ChildContainer {
            wrapper: AbstractChildWrapper::new(child.clone(), cell_settings),
            row,
            column,
            occupied_rows: rows,
            occupied_columns: columns,
        });
        child
    }

    pub fn column_spacing(&mut self, column_spacing: i32) -> &mut GridLayout {
        self.column_spacing = column_spacing;
        self
    }

    pub fn row_spacing(&mut self, row_spacing: i32) -> &mut GridLayout {
        self.row_spacing = row_spacing;
        self
    }

    pub fn spacing(&mut self, spacing: i32) -> &mut GridLayout {
        self.column_spacing(spacing).row_spacing(spacing)
    }

    pub fn new_cell_settings(&self) -> LayoutSettings {
        self.default_cell_settings.copy()
    }

    pub fn default_cell_setting(&self) -> LayoutSettings {
        self.default_cell_settings.clone()
    }

    pub fn create_row_helper(&self, columns: i32) -> RowHelper {
        RowHelper { columns, index: 0 }
    }

    fn child_refs(&self) -> Vec<LayoutElementRef> {
        self.children
            .iter()
            .map(|child| child.wrapper.child.clone())
            .collect()
    }
}

impl LayoutElement for GridLayout {
    fn set_x(&mut self, x: i32) {
        let children = self.child_refs();
        self.base.set_x(x, children);
    }

    fn set_y(&mut self, y: i32) {
        let children = self.child_refs();
        self.base.set_y(y, children);
    }

    fn get_x(&self) -> i32 {
        self.base.get_x()
    }

    fn get_y(&self) -> i32 {
        self.base.get_y()
    }

    fn get_width(&self) -> i32 {
        self.base.width
    }

    fn get_height(&self) -> i32 {
        self.base.height
    }
}

impl Layout for GridLayout {
    fn visit_children(&self, layout_element_visitor: &mut dyn FnMut(&LayoutElementRef)) {
        for child in &self.children {
            layout_element_visitor(&child.wrapper.child);
        }
    }

    fn arrange_elements(&mut self) {
        layout::arrange_children(self);
        let mut max_row = 0;
        let mut max_column = 0;
        for child in &self.children {
            max_row = child.get_last_occupied_row().max(max_row);
            max_column = child.get_last_occupied_column().max(max_column);
        }

        let mut max_column_widths = vec![0; (max_column + 1) as usize];
        let mut max_row_heights = vec![0; (max_row + 1) as usize];
        for child in &self.children {
            let child_height =
                child.wrapper.get_height() - (child.occupied_rows - 1) * self.row_spacing;
            let mut height_divisor = Divisor::new(child_height, child.occupied_rows);
            for row in child.row..=child.get_last_occupied_row() {
                let row = row as usize;
                max_row_heights[row] = max_row_heights[row].max(height_divisor.next_int());
            }

            let child_width =
                child.wrapper.get_width() - (child.occupied_columns - 1) * self.column_spacing;
            let mut width_divisor = Divisor::new(child_width, child.occupied_columns);
            for column in child.column..=child.get_last_occupied_column() {
                let column = column as usize;
                max_column_widths[column] = max_column_widths[column].max(width_divisor.next_int());
            }
        }

        let mut column_x_offsets = vec![0; (max_column + 1) as usize];
        let mut row_y_offsets = vec![0; (max_row + 1) as usize];
        for column in 1..=max_column as usize {
            column_x_offsets[column] =
                column_x_offsets[column - 1] + max_column_widths[column - 1] + self.column_spacing;
        }
        for row in 1..=max_row as usize {
            row_y_offsets[row] =
                row_y_offsets[row - 1] + max_row_heights[row - 1] + self.row_spacing;
        }

        for child in &self.children {
            let mut available_width = 0;
            for column in child.column..=child.get_last_occupied_column() {
                available_width += max_column_widths[column as usize];
            }
            available_width += self.column_spacing * (child.occupied_columns - 1);
            child.wrapper.set_x(
                self.base.get_x() + column_x_offsets[child.column as usize],
                available_width,
            );

            let mut available_height = 0;
            for row in child.row..=child.get_last_occupied_row() {
                available_height += max_row_heights[row as usize];
            }
            available_height += self.row_spacing * (child.occupied_rows - 1);
            child.wrapper.set_y(
                self.base.get_y() + row_y_offsets[child.row as usize],
                available_height,
            );
        }

        self.base.width =
            column_x_offsets[max_column as usize] + max_column_widths[max_column as usize];
        self.base.height = row_y_offsets[max_row as usize] + max_row_heights[max_row as usize];
    }

    fn remove_children(&mut self) {
        self.children.clear();
    }
}

/// `GridLayout.RowHelper`: children one after another, `columns` to a row. Its grid is a
/// parameter, Java's outer instance.
pub struct RowHelper {
    columns: i32,
    index: i32,
}

impl RowHelper {
    pub fn add_child(
        &mut self,
        grid: &mut GridLayout,
        widget: LayoutElementRef,
    ) -> LayoutElementRef {
        self.add_child_columns(grid, widget, 1)
    }

    /// `addChild(T, int)`.
    pub fn add_child_columns(
        &mut self,
        grid: &mut GridLayout,
        widget: LayoutElementRef,
        column_width: i32,
    ) -> LayoutElementRef {
        let settings = grid.default_cell_setting();
        self.add_child_columns_with(grid, widget, column_width, settings)
    }

    /// `addChild(T, LayoutSettings)`.
    pub fn add_child_with(
        &mut self,
        grid: &mut GridLayout,
        widget: LayoutElementRef,
        layout_settings: LayoutSettings,
    ) -> LayoutElementRef {
        self.add_child_columns_with(grid, widget, 1, layout_settings)
    }

    /// `addChild(T, int, LayoutSettings)`.
    pub fn add_child_columns_with(
        &mut self,
        grid: &mut GridLayout,
        widget: LayoutElementRef,
        column_width: i32,
        layout_settings: LayoutSettings,
    ) -> LayoutElementRef {
        let mut row = self.index / self.columns;
        let mut column_begin = self.index % self.columns;
        if column_begin + column_width > self.columns {
            row += 1;
            column_begin = 0;
            self.index = Mth::round_toward_i32(self.index, self.columns);
        }
        self.index += column_width;
        grid.add_child_spanning_with(widget, row, column_begin, 1, column_width, layout_settings)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use serde_json::{Value, json};

    use super::GridLayout;
    use crate::golden::Golden;
    use crate::net::minecraft::client::gui::layouts::frame_layout::FrameLayout;
    use crate::net::minecraft::client::gui::layouts::layout_element::LayoutElementRef;
    use crate::net::minecraft::client::gui::layouts::layout_settings::LayoutSettings;
    use crate::net::minecraft::client::gui::layouts::linear_layout::LinearLayout;
    use crate::net::minecraft::client::gui::layouts::spacer_element::SpacerElement;

    fn int(value: &Value) -> i32 {
        value.as_i64().expect("an int") as i32
    }

    fn float(value: &Value) -> f32 {
        f32::from_bits(
            u32::from_str_radix(value.as_str().expect("a float's bits"), 16).expect("hex"),
        )
    }

    /// The spec's settings over `settings`.
    fn apply(settings: &LayoutSettings, spec: &Value) -> LayoutSettings {
        settings
            .padding_ltrb(int(&spec[0]), int(&spec[1]), int(&spec[2]), int(&spec[3]))
            .align(float(&spec[4]), float(&spec[5]))
    }

    /// Builds the suite's tree (its javadoc has the format), keeping its spacers in order.
    fn build(spec: &Value, spacers: &mut Vec<Rc<RefCell<SpacerElement>>>) -> LayoutElementRef {
        let children = |index: usize| spec[index].as_array().expect("children").clone();
        match spec[0].as_str().expect("a kind") {
            "spacer" => {
                let spacer = Rc::new(RefCell::new(SpacerElement::new(
                    int(&spec[1]),
                    int(&spec[2]),
                )));
                spacers.push(Rc::clone(&spacer));
                LayoutElementRef::Element(spacer)
            }
            "grid" => {
                let mut grid = GridLayout::new();
                grid.row_spacing(int(&spec[1]))
                    .column_spacing(int(&spec[2]));
                apply(&grid.default_cell_setting(), &spec[3]);
                for child in children(4) {
                    let element = build(&child[0], spacers);
                    let settings = match &child[5] {
                        Value::Null => grid.new_cell_settings(),
                        settings => apply(&grid.new_cell_settings(), settings),
                    };
                    grid.add_child_spanning_with(
                        element,
                        int(&child[1]),
                        int(&child[2]),
                        int(&child[3]),
                        int(&child[4]),
                        settings,
                    );
                }
                LayoutElementRef::layout(Rc::new(RefCell::new(grid)))
            }
            "rows" => {
                let mut grid = GridLayout::new();
                apply(&grid.default_cell_setting(), &spec[2]);
                let mut helper = grid.create_row_helper(int(&spec[1]));
                for child in children(3) {
                    let element = build(&child[0], spacers);
                    match &child[2] {
                        Value::Null => {
                            helper.add_child_columns(&mut grid, element, int(&child[1]));
                        }
                        settings => {
                            let settings = apply(&grid.new_cell_settings(), settings);
                            helper.add_child_columns_with(
                                &mut grid,
                                element,
                                int(&child[1]),
                                settings,
                            );
                        }
                    }
                }
                if !spec[4].is_null() {
                    apply(&grid.default_cell_setting(), &spec[4]);
                }
                LayoutElementRef::layout(Rc::new(RefCell::new(grid)))
            }
            "linear" => {
                let base = if spec[1].as_bool().expect("vertical") {
                    LinearLayout::vertical()
                } else {
                    LinearLayout::horizontal()
                };
                let mut linear = base.spacing(int(&spec[2]));
                for child in children(3) {
                    let element = build(&child[0], spacers);
                    let settings = match &child[1] {
                        Value::Null => linear.new_cell_settings(),
                        settings => apply(&linear.new_cell_settings(), settings),
                    };
                    linear.add_child_with(element, settings);
                }
                LayoutElementRef::layout(Rc::new(RefCell::new(linear)))
            }
            "frame" => {
                let mut frame = FrameLayout::with_min(int(&spec[1]), int(&spec[2]));
                for child in children(3) {
                    let element = build(&child[0], spacers);
                    let settings = match &child[1] {
                        Value::Null => frame.new_child_layout_settings(),
                        settings => apply(&frame.new_child_layout_settings(), settings),
                    };
                    frame.add_child_with(element, settings);
                }
                LayoutElementRef::layout(Rc::new(RefCell::new(frame)))
            }
            kind => panic!("unknown node {kind}"),
        }
    }

    /// The layouts' math: 600 random trees of grids, row helpers, linear layouts and frames over
    /// spacers, arranged and aligned in a rectangle (TESTING.md, "The GUI").
    #[test]
    fn arranges_layouts_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.gui.layouts.GridLayout");
        let cases = golden.cases("arrange");
        assert_eq!(cases.len(), 600);
        let mut failures = Vec::new();
        for (args, expected) in cases {
            let mut spacers = Vec::new();
            let root = build(&args[0], &mut spacers);
            if let Some(layout) = root.as_layout() {
                layout.borrow_mut().arrange_elements();
            }
            FrameLayout::align_in_rectangle(
                &root,
                int(&args[1]),
                int(&args[2]),
                int(&args[3]),
                int(&args[4]),
                float(&args[5]),
                float(&args[6]),
            );
            let places: Vec<Value> = spacers
                .iter()
                .map(|spacer| {
                    use crate::net::minecraft::client::gui::layouts::layout_element::LayoutElement;
                    let spacer = spacer.borrow();
                    json!([spacer.get_x(), spacer.get_y()])
                })
                .collect();
            let actual = json!([
                [
                    root.get_x(),
                    root.get_y(),
                    root.get_width(),
                    root.get_height()
                ],
                places
            ]);
            if actual != expected && failures.len() < 3 {
                failures.push(format!(
                    "{}\n expected {expected}\n got      {actual}",
                    args[0]
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        golden.finish();
    }
}
