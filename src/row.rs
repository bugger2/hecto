use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use crate::editor::TAB_WIDTH;

#[derive(Default)]
pub struct Row {
	string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut ret = Row {
            string: String::from(slice),
            len: 0,
        };
        ret.update_len();
        ret
    }
}

impl Row {
    #[must_use] pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        // self.string.get(start..end).unwrap_or_default().to_string()
        let mut ret = String::new();
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end-start)
        {
            if grapheme == "\t" {
                ret.push_str(&" ".repeat(TAB_WIDTH as usize) as &str);
            } else {
                ret.push_str(grapheme);
            }
        }
        ret
    }

    pub fn push(&mut self, c: char) {
        if c != '\t' {
            self.string.push(c);
        } else {
            self.string.push_str(&" ".repeat(TAB_WIDTH as usize));
        }
        self.update_len();
    }

    pub fn insert(&mut self, index: usize, c: char) {
        if c != '\t' {
            self.string.insert(index, c);
        } else {
            self.string.insert_str(index, &" ".repeat(TAB_WIDTH as usize));
        }
        self.update_len();
    }

    #[must_use] pub fn len(&self) -> usize {
        self.len
    }

    #[must_use] pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn char_count(&self, character: char) -> usize {
        let mut ret = 0;
        for c in self.string.chars() {
            if c == character {
                ret += 1;
            }
        }
        ret
    }

    fn update_len(&mut self) {
        self.len = self.string.graphemes(true).count().saturating_add(self.char_count('\t') * (TAB_WIDTH.saturating_sub(1) as usize));
    }
}
