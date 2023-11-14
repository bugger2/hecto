use crate::Document;
use crate::Row;
use crate::terminal;
use std::io;
use std::env;
use termion::event::Key;
use terminal::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
	pub x: usize,
	pub y: usize,
}

pub struct Editor {
	should_quit: bool,
	terminal: Terminal,
	cursor_position: Position,
	document: Document,
}

impl Editor {
	pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let filename = &args[1];
            Document::open(&filename).unwrap_or_default()
        } else {
            Document::default()
        };
		Self {
			should_quit: false,
			terminal: Terminal::new().expect("Failed to initialize terminal"),
			cursor_position: Position::default(),
			document,
		}
	}

	pub fn run(&mut self) {
		if let Err(error) = self.refresh_screen() {
			die(&error);
		}
		
		loop {
			if let Err(error) = self.process_keypress() {
				die(&error);
			}

			if let Err(error) = self.refresh_screen() {
				die(&error);
			}

			if self.should_quit {
				break;
			}
		}
	}


	fn process_keypress(&mut self) -> Result<(), std::io::Error> {
		let key_pressed = Terminal::read_key()?;
		match key_pressed {
			Key::Ctrl('q') => self.should_quit = true,
			Key::Left
				| Key::Right
				| Key::Up
				| Key::Down
				| Key::Ctrl('n')
				| Key::Ctrl('p')
				| Key::Ctrl('b')
				| Key::Ctrl('f')
				| Key::Home
				| Key::End
				| Key::PageUp
				| Key::PageDown => self.move_cursor(key_pressed),
			_ => (),
		}
		Ok(())
	}

	fn move_cursor(&mut self, key: Key) {
		let mut x = self.cursor_position.x;
		let mut y = self.cursor_position.y;
		let width = self.terminal.size().width as usize;
		let height = self.terminal.size().height as usize;
		match key {
			Key::Left | Key::Ctrl('b') => if x > 0 {x = x.saturating_sub(1)},
			Key::Right | Key::Ctrl('f') => if x < width {x = x.saturating_add(1)},
			Key::Up | Key::Ctrl('p') => if y > 0 {y = y.saturating_sub(1)},
			Key::Down | Key::Ctrl('n') => if y < height {y = y.saturating_add(1)},
			Key::Home => y = 0,
			Key::End => y = height,
			_ => (),
		}
		self.cursor_position = Position { x, y };
	}

    pub fn draw_row(&self, row: &Row) {
        let start = 0;
        let end = self.terminal.size().width as usize;
        let row = row.render(start, end);
        println!("{}\r", row);
    }

	fn draw_rows(&self) {
		Terminal::cursor_position(&Position::default());
		let height = self.terminal.size().height;
		for terminal_row in 0..height-1 {
			Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
				self.draw_welcome_message();
			} else {
				println!("~\r");
			}
		}
	}

	fn draw_welcome_message(&self) {
		let mut welcome_message = format!("Hecto Editor -- Version {VERSION}");
		let width = self.terminal.size().width as usize;
		let len = welcome_message.len();
		let padding = width.saturating_sub(len) / 2;
		let spaces = " ".repeat(padding.saturating_sub(1));
		welcome_message = format!("~{spaces}{welcome_message}");
		welcome_message.truncate(width);
		println!("{welcome_message}\r");
	}

	fn refresh_screen(&self) -> Result<(), std::io::Error> {
		Terminal::hide_cursor();
		Terminal::cursor_position(&self.cursor_position);
		if self.should_quit {
			Terminal::clear_screen();
			println!("Goodbye! \r");
		} else {
			self.draw_rows();
			Terminal::cursor_position(&self.cursor_position);
		}
		Terminal::show_cursor();
		Terminal::flush()
	}
}


fn die(e: &io::Error) {
	Terminal::clear_screen();
	panic!("{}", e);
}
