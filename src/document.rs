use crate::{row::Row, editor::Position};
use std::fs;

#[derive(Default)]
pub struct Document {
	rows: Vec<Row>,
    pub filename: Option<String>,
}

impl Document {

    /// # Errors
    ///
    /// If the file cannot be read (permissions denied, file doesn't exist, etc.) then the error
    /// will be propagated
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
        })
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y == self.len() {
            let mut row = Row::default();
            row.push(c);
            self.rows.push(row);
        } else {
            let row: &mut Row = self.rows.get_mut(at.y).unwrap();
            if at.x == row.len() {
                row.push(c);
            } else {
                row.insert(at.x, c);
            }
        }
    }

    pub fn del_char_backward(&mut self, at: &Position) {
        let empty_row_mut = &mut Row::default();
        if at.x != 0 {
            let row: &mut Row = self.rows.get_mut(at.y).unwrap_or(empty_row_mut);
            row.delete(at.x.saturating_sub(1));
        } else if at.y > 0 {
            let curr_row_contents = self.row(at.y).unwrap_or(&Row::default()).contents();

            let prev_row: &mut Row = self.rows.get_mut(at.y-1).unwrap_or(empty_row_mut);
            prev_row.push_str(&curr_row_contents);

            if at.y < self.rows.len() {
                self.rows.remove(at.y);
            }
        }
    }

    pub fn del_char_forward(&mut self, at: &Position) {
        let empty_row_mut = &mut Row::default();
        let row: &mut Row = self.rows.get_mut(at.y).unwrap_or(empty_row_mut);
        if at.x != row.len() {
            row.delete(at.x);
        } else if at.y < self.len() {
            let next_row_contents = self.row(at.y.saturating_add(1)).unwrap_or(&Row::default()).contents();
            let empty_row_mut = &mut Row::default();

            let curr_row: &mut Row = self.rows.get_mut(at.y).unwrap_or(empty_row_mut);
            curr_row.push_str(&next_row_contents);

            if at.y.saturating_add(1) < self.rows.len() {
                self.rows.remove(at.y.saturating_add(1));
            }
        }
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y >= self.len() {
            self.rows.push(Row::default());
            self.rows.push(Row::default());
        } else if at.x == self.row(at.y).unwrap_or(&Row::default()).len() {
            self.rows.insert(at.y.saturating_add(1), Row::default());
        } else {
            let empty_row_mut = &mut Row::default();

            let curr_row = self.rows.get_mut(at.y).unwrap_or(empty_row_mut);
            let curr_row_contents = curr_row.contents();

            let split_content = curr_row_contents.split_at(at.x);

            let mut new_row = Row::default();

            new_row.push_str(split_content.1);
            curr_row.clear_mut().push_str(split_content.0);

            self.rows.insert(at.y.saturating_add(1), new_row);
        }
    }

    #[must_use] pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[must_use] pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

	#[must_use] pub fn len(&self) -> usize {
		self.rows.len()
	}
}
