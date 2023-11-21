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
