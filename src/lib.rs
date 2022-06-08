
use termchars::TermString;
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug)]
pub enum Pos {
    Left,
    Middle,
    Right
}

#[derive(Copy, Clone, Debug)]
pub struct ColumnLayout {
    lower_bound: usize,
    upper_bound: Option<usize>,
    pos: Pos,
    pad_char: char
}

impl ColumnLayout {

    pub fn align(pos: Pos, pad_char: char) -> ColumnLayout {
        ColumnLayout { lower_bound: 0
                     , upper_bound: None
                     , pos
                     , pad_char
                     }
    }

    pub fn fixed_width(width: usize, pad_char: char) -> ColumnLayout {
        ColumnLayout { lower_bound: width
                     , upper_bound: Some(width)
                     , pos: Pos::Right
                     , pad_char
                     }
    }

    pub fn repeat(&self, count: usize) -> Vec<ColumnLayout> {
        vec![self.clone(); count]
    }

    pub fn set_lower_bound(&mut self, lower_bound: usize) {
        self.lower_bound = lower_bound;
    }

    pub fn set_upper_bound(&mut self, upper_bound: usize) {
        self.upper_bound = Some(upper_bound);
    }

    pub fn eliminate_upper_bound(&mut self) {
        self.upper_bound = None;
    }

    pub fn set_pos(&mut self, pos: Pos) {
        self.pos = pos;
    }

    pub fn set_pad_char(&mut self, pad_char: char) {
        self.pad_char = pad_char;
    }

    fn render(&self, min: usize, max: usize, value: &str, out: &mut String) {
        if min > max {
            panic!("min > max");
        }

        let text = TermString::new(value, false).unwrap();
        let count = text.clone().visible_chars_count();

        let (clone, pads_needed) = match self.upper_bound {
            None => {
                (text.truncated(count).to_string(), max - count)
            }
            Some(bound) => {
                let needed = if count > self.lower_bound {
                    0
                } else {
                    self.lower_bound - count
                };

                if count > bound {
                    (text.truncated(bound).to_string(), needed)
                } else {
                    (text.truncated(count).to_string(), needed)
                }
            }
        };

        match self.pos {
            Pos::Right => {
                for _ in 0..pads_needed {
                    out.push(self.pad_char);
                }
                out.push_str(clone.as_str())
            }
            Pos::Left => {
                out.push_str(clone.as_str());
                for _ in 0..pads_needed {
                    out.push(self.pad_char);
                }
            }
            Pos::Middle => {
                let pad_count = pads_needed/2;
                for _ in 0..pad_count {
                    out.push(self.pad_char);
                }
                out.push_str(clone.as_str());
                for _ in 0..pad_count {
                    out.push(self.pad_char);
                }
                if pad_count % 2 == 1 {
                    out.push(self.pad_char);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RowLayout {
    start: String,
    end:   String,
    sep:   String,
    columns: Vec<ColumnLayout>
}

impl RowLayout {
    pub fn new() -> RowLayout {
        RowLayout { start: "".to_string()
                  , end:   "".to_string()
                  , sep:   "".to_string()
                  , columns: vec![]
                  }
    }

    pub fn set_start_token(&mut self, token: String) {
        self.start = token;
    }

    pub fn set_end_token(&mut self, token: String) {
        self.end = token;
    }

    pub fn set_separator(&mut self, token: String) {
        self.sep = token;
    }

    pub fn push_column(&mut self, column: ColumnLayout) {
        self.columns.push(column)
    }

    pub fn extend_columns(&mut self, columns: &[ColumnLayout]) {
        self.columns.extend(columns)
    }
}

pub struct Renderer {
    rules: Vec<(RowLayout, Vec<(usize, usize, VecDeque<String>)>)>,
    newline: String,
    begin: String,
    end: String,
    write_logs: VecDeque<usize>
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { rules: vec![]
                 , newline: "\n".to_string()
                 , begin: "".to_string()
                 , end: "".to_string()
                 , write_logs: VecDeque::new() 
                 }
    }

    pub fn set_newline(&mut self, newline: String) {
        self.newline = newline
    }

    pub fn set_begin(&mut self, begin: String) {
        self.begin = begin;
    }

    pub fn set_end(&mut self, end: String) {
        self.end = end;
    }

    pub fn register_layout(&mut self, layout: RowLayout) -> usize {
        let new_id  = self.rules.len();
        let ins_dat = vec![(usize::MAX,0,VecDeque::new());layout.columns.len()];
        self.rules.push((layout.clone(), ins_dat));
        new_id
    }

    pub fn write_to_layout(&mut self, layout: usize, data: &[String]) {
        if let Some((def, cols_dat)) = self.rules.get_mut(layout) {
            // check if the dimential matches between the column definiton and
            // input data
            if def.columns.len() == data.len() && cols_dat.len() == data.len() {
                for (i, (lowest, highest, values)) in cols_dat.iter_mut().enumerate() {
                    let text = TermString::new(&data[i], false).unwrap();
                    let text_len = text.visible_chars_count();
                    *lowest = std::cmp::min(*lowest, text_len);
                    *highest = std::cmp::max(*highest, text_len);
                    (*values).push_back(data[i].clone());
                }

                self.write_logs.push_back(layout)
            }
        }
    }

    pub fn flush(&mut self) -> String {
        let mut buf = String::new();
        let mut is_fst_line = false;
        while let Some(rule_idx) = self.write_logs.pop_front() {
            if is_fst_line {
                buf.push_str(self.newline.as_str());
            } else {
                is_fst_line = true;
            }
            let (def, cols_dat) = self.rules.get_mut(rule_idx).unwrap();
            let mut once = false;
            buf.push_str(def.start.as_str());
            // Iterate the state and definition together
            let zipped = cols_dat.iter_mut().zip(def.columns.iter());
            // For each column in the row
            for ((min, max, deque), col) in zipped {
                if once {
                    buf.push_str(def.sep.as_str());
                } else {
                    once = true;
                }
                col.render(*min, *max, &deque.pop_front().expect(""), &mut buf);
            }
            buf.push_str(def.end.as_str());
        }
        buf
    }
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_multi_format() {
        let col1 = ColumnLayout::align(Pos::Right, '0');
        let col2 = ColumnLayout::fixed_width(5, '-');
        let col3 = ColumnLayout::fixed_width(5, ' ');
        let mut row1 = RowLayout::new();
        row1.set_separator("|".to_string());
        row1.push_column(col1);
        row1.push_column(col3);

        let mut row2 = RowLayout::new();
        row2.set_separator("|".to_string());
        row2.push_column(col2);
        row2.push_column(col3);

        let mut r = Renderer::new();
        let h1 = r.register_layout(row1);
        let h2 = r.register_layout(row2);

        r.write_to_layout(h1, &["hello".to_string(), "world".to_string()]);
        r.write_to_layout(h2, &["does-it".to_string(), "work".to_string()]);
        r.write_to_layout(h1, &["longlong".to_string(), "a".to_string()]);

        let emit = r.flush();

        assert_eq!(emit.as_str(),
          "000hello|world\ndoes-| work\nlonglong|    a");
    }

    #[test]
    fn test_column_rendering() {
        let column = ColumnLayout
            { lower_bound: 0
            , upper_bound: None
            , pos: Pos::Left
            , pad_char: ' '
            };

        {
            let mut buffer = String::new();
            column.render(0, 8, &"meow", &mut buffer);
            assert_eq!(buffer.as_str(), "meow    ");
        }
    }

    #[test]
    fn test_column_middle_pos() {
        let column = ColumnLayout
            { lower_bound: 0
            , upper_bound: None
            , pos: Pos::Middle
            , pad_char: ' '
            };

        {
            let mut buffer = String::new();
            column.render(0, 8, &"meow", &mut buffer);
            assert_eq!(buffer.as_str(), "  meow  ");
        }
    }

    #[test]
    fn test_fixed_width_columns() {
        let col = ColumnLayout::fixed_width(5, ' ');
        let mut row = RowLayout::new();
        row.set_separator(" ".to_string());
        row.extend_columns(&col.repeat(3));
        let mut renderer = Renderer::new();
        let handle = renderer.register_layout(row);
        renderer.write_to_layout(
            handle,
            &["123456".to_string(), "1".to_string(), "\x1b[93m12345\x1b[0m".to_string()]);
        renderer.write_to_layout(
            handle,
            &["1".to_string(), "123".to_string(), "".to_string()]);
        let output = renderer.flush();
        assert_eq!(output.as_str(),
            "12345     1 \x1b[93m12345\x1b[0m\n    1   123      ");
    }

    #[test]
    fn test_rows_rendering() {
        let unbound_col = ColumnLayout::align(Pos::Left, ' ');
        let mut row = RowLayout::new();
        row.set_separator(", ".to_string());
        row.set_start_token("[".to_string());
        row.set_end_token("]".to_string());
        row.extend_columns(&unbound_col.repeat(3));

        let mut renderer = Renderer::new();
        let handle = renderer.register_layout(row);

        renderer.write_to_layout(
            handle,
            &["sheep".to_string(), "bmw".to_string(), "malloc".to_string(),
        ]);

        let output = renderer.flush();

        assert_eq!(output.as_str(), "[sheep, bmw, malloc]");

        renderer.write_to_layout(
            handle, 
            &["sheep".to_string(), "bmw".to_string(), "malloc".to_string()]);

        renderer.write_to_layout(
            handle,
            &["12345678".to_string(), "1".to_string(), "\x1b[93mmalloc\x1b[0m".to_string()]);

        let output = renderer.flush();

        assert_eq!(output.as_str(),
          "[sheep   , bmw, malloc]\n[12345678, 1  , \x1b[93mmalloc\x1b[0m]".to_string());
    }
}
