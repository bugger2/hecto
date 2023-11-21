use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

const TAB_WIDTH: u32 = 4;

pub struct Row {
	string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let ret = Row {
            string: String::from(slice),
            len: 0,
        };
        ret.update_len()
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
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

    pub fn len(&self) -> usize {
        self.len
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

    fn update_len(&self) -> Self {
        Row {
            string: self.string.clone(),
            len: self.string.graphemes(true).count().saturating_add(self.char_count('\t') * (TAB_WIDTH.saturating_sub(1) as usize)),
        }
    }
}
