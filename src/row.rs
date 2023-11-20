use std::cmp;

pub struct Row {
	string: String,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Row {
            string: String::from(slice),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        self.string.get(start..end).unwrap_or_default().to_string()
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }

    pub fn char_count(&self, character: char) -> u32 {
        let mut ret: u32 = 0;
        for i in self.string.chars() {
            if i == character { ret+=1; }
        }
        ret
    }
}
