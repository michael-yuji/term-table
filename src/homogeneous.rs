use super::*;
use std::collections::HashMap;

pub trait TableSource {
    fn value_for_column(&self, column: &str) -> Option<String>;
}

pub struct TableLayout {
    permutation: Vec<String>,
    renderer: Renderer,
    row_id: usize,
    print_header: bool
}

impl TableLayout {
    pub fn new(
        separator: &str,
        print_header: bool,
        layout: Vec<(String, ColumnLayout)>
    ) -> TableLayout
    {
        let mut permutation = Vec::new();
        let mut row_layout = RowLayout::new();
        row_layout.set_separator(separator.to_string());
        for (title, layout) in layout.into_iter() {
            row_layout.push_column_layout(layout);
            permutation.push(title);
        }

        let mut renderer = Renderer::new();
        let row_id = renderer.register_layout(row_layout);

        if print_header {
            renderer.write_to_layout(row_id, &permutation);
        }

        TableLayout {
            permutation,
            renderer,
            row_id,
            print_header
        }
    }

    pub fn append_data(&mut self, source: impl TableSource) {
        let mut dat = Vec::new();
        for key in self.permutation.iter() {
            match source.value_for_column(key.as_str()) {
                None => dat.push("N/A".to_string()),
                Some(value) => dat.push(value)
            }
        }
        self.renderer.write_to_layout(self.row_id, &dat);
    }

    pub fn flush(&mut self) -> String {
        let ret = self.renderer.flush();
        if self.print_header {
            self.renderer.write_to_layout(self.row_id, &self.permutation);
        }
        ret
    }
}
