use std::io::{self, stdout, Stdout, Write};
use termion::{raw::{IntoRawMode, RawTerminal}, event::Key, input::TermRead};

use crate::editor::Position;
	
pub struct Size {
	pub width: u16,
	pub height: u16,
}

pub struct Terminal {
	size: Size,
	_stdout: RawTerminal<Stdout>,
}

impl Terminal {

	/// # Panics
	/// 
	/// Will panic if unable to open stdout in raw mode
	///
	/// # Errors
	///
	/// Will return an error if unable to determine terminal dimensions
	pub fn new() -> Result<Self, std::io::Error> {
		let size = termion::terminal_size()?;
		Ok(Terminal {
			size: Size {
				width: size.0,
				height: size.1,
			},
			_stdout: stdout().into_raw_mode().unwrap(),
		})
	}

	/// # Errors
	///
	/// Will error if unable to retrieve the next key press
	pub fn read_key() -> Result<Key, std::io::Error> {
		loop {
			if let Some(key) = io::stdin().lock().keys().next() {
				return key;
			}
		}
	}

	#[must_use] pub fn size(&self) -> &Size {
		&self.size
	}

	pub fn clear_screen() {
		print!("{}", termion::clear::All);
	}

	#[allow(clippy::cast_possible_truncation)]
	pub fn cursor_position(position: &Position) {
		let x = position.x.saturating_add(1) as u16;
		let y = position.y.saturating_add(1) as u16;

		print!("{}", termion::cursor::Goto(x, y));
	}

	/// # Errors
	///
	/// Will error if cannot flush stdout
	pub fn flush() -> Result<(), io::Error> {
		io::stdout().flush()
	}

	pub fn hide_cursor() {
		print!("{}", termion::cursor::Hide);
	}

	pub fn show_cursor() {
		print!("{}", termion::cursor::Show);
	}

	pub fn clear_current_line() {
		print!("{}", termion::clear::CurrentLine);
	}
}
