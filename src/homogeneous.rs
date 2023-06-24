use super::*;

pub trait TextFormatter {
    fn format_text(&self, source: String) -> String;
}

pub struct IdFormatter();

impl TextFormatter for IdFormatter {
    fn format_text(&self, source: String) -> String {
        source
    }
}

pub struct Title {
    pub title: String,
    pub id: String,
    pub formatter: Box<dyn TextFormatter>
}

impl Title {
    pub fn new(title: &str, id: &str) -> Title {
        Title {
            title: title.to_string(),
            id: id.to_string(),
            formatter: Box::new(IdFormatter())
        }
    }
    pub fn new_fmt(title: &str, id: &str, formatter: impl TextFormatter + 'static) -> Title {
        Title {
            title: title.to_string(),
            id: id.to_string(),
            formatter: Box::new(formatter)
        }
    }
}

pub trait TableSource {
    fn value_for_column(&self, column: &str) -> Option<String>;
}

pub struct TableLayout {
    permutation: Vec<Title>,
    renderer: Renderer,
    row_id: usize,
    print_header: bool,
    // cache
    titles: Vec<String>
}

impl TableLayout {
    pub fn new(
        separator: &str,
        print_header: bool,
        layout: Vec<(Title, ColumnLayout)>
    ) -> TableLayout
    {
        let mut permutation = Vec::new();
        let mut titles = Vec::new();
        let mut row_layout = RowLayout::new();
        row_layout.set_separator(separator.to_string());
        for (title, layout) in layout.into_iter() {
            row_layout.push_column_layout(layout);
            titles.push(title.title.to_string());
            permutation.push(title);
        }

        let mut renderer = Renderer::new();
        let row_id = renderer.register_layout(row_layout);

        if print_header {
            renderer.write_to_layout(row_id, &titles);
        }

        TableLayout {
            permutation,
            renderer,
            row_id,
            print_header,
            titles
        }
    }

    pub fn append_data(&mut self, source: impl TableSource) {
        let mut dat = Vec::new();
        for title in self.permutation.iter() {
            let value = match source.value_for_column(&title.id) {
                None => "N/A".to_string(),
                Some(value) => value
            };
            let formatted = title.formatter.format_text(value);
            dat.push(formatted);
        }
        self.renderer.write_to_layout(self.row_id, &dat);
    }

    pub fn flush(&mut self) -> String {
        let ret = self.renderer.flush();
        if self.print_header {
            self.renderer.write_to_layout(self.row_id, &self.titles);
        }
        ret
    }
}
