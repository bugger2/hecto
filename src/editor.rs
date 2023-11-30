// hello from hecto

use crate::Document;
use crate::Row;
use crate::terminal;
use std::io;
use std::env;
use core::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;
use terminal::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239); // #EFEFEF
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63); // #3F3F3F
pub const TAB_WIDTH: u32 = 4;

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct StatusMessage {
    message: String,
    timestamp: Instant,
}
impl From<String> for StatusMessage {
    fn from(message: String) -> StatusMessage {
        StatusMessage {
            message,
            timestamp: Instant::now(),
        }
    }
}
impl From<&str> for StatusMessage {
    fn from(message: &str) -> StatusMessage {
        StatusMessage::from(message.to_owned())
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
    dirty: bool,
}

impl Editor {
    pub fn default() -> Self {
        let mut initial_status = String::from("Help: Ctrl-s to search | Ctrl-w to save | Ctrl-q to exit");
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let filename = &args[1];
            let doc = Document::open(filename);
            if let Ok(document) = doc {
                document
            } else {
                initial_status = format!("ERROR: Failed to open file {filename}");
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::new().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            dirty: false,
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
            Key::Ctrl('w') => self.save()
                .unwrap_or_else(|_| println!("ERROR: Failed to save {filename}",
                                             filename = self.document.filename.clone().unwrap_or(String::from("file")))),
            Key::Ctrl('s') => self.find()?,
            Key::Char(c) => self.insert_char(c),
            Key::Backspace => self.del_char_backward(),
            Key::Delete => self.del_char_forward(),
            | Key::Left
                | Key::Right
                | Key::Up
                | Key::Down
                | Key::Ctrl('n' | 'p' | 'b' | 'f' | 'e' | 'a')
                | Key::Home
                | Key::End
                | Key::PageUp
                | Key::PageDown => self.move_cursor(key_pressed),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn save(&mut self) -> Result<(), io::Error> {
        if self.document.filename.is_none() {
            let new_name = self.prompt_string("Save as: ", |_, _, _| {})?;
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.");
                return Ok(());
            }
            self.document.filename = new_name;
        }

        self.document.save()?;
        self.status_message = StatusMessage::from(format!("Successfully saved {}", self.document.filename.clone().unwrap_or(String::from("file"))));
        self.dirty = false;
        Ok(())
    }

    fn find(&mut self) -> Result<(), io::Error> {
        let initial_position = self.cursor_position.clone();

        if let Some(query) = self.prompt_string("Search: ", |editor, _, query| {
            if let Some(position) = editor.document.find(query) {
                editor.cursor_position = position;
                editor.scroll();
            }})?
        {
            if let Some(position) = self.document.find(&query) {
                self.cursor_position = position;
            } else {
                self.status_message = StatusMessage::from(format!("Not found: {query}"));
            }
        } else {
            self.cursor_position = initial_position;
            self.scroll();
        }
        Ok(())
    }

    fn insert_char(&mut self, c: char) {
        self.dirty = true;
        if c != '\n' {
            self.document.insert(&self.cursor_position, c);
        } else {
            self.document.insert_newline(&self.cursor_position);
        }

        // handling cursor position
        let x = &mut self.cursor_position.x;
        if c == '\t' {
            *x = x.saturating_add(TAB_WIDTH as usize);
        } else if c == '\n' {
            self.cursor_position.y += 1;
            self.cursor_position.x = 0;
        } else {
            *x = x.saturating_add(1);
        }
    }

    fn del_char_backward(&mut self) {
        self.dirty = true;
        let prev_line_len = self.document.row(self.cursor_position.y.saturating_sub(1)).unwrap_or(&Row::default()).len();
        self.document.del_char_backward(&self.cursor_position);
        let x = &mut self.cursor_position.x;
        let y = &mut self.cursor_position.y;
        if *x != 0 {
            *x -= 1;
        } else if y > &mut 0 {
            *x = prev_line_len;
            *y -= 1;
        }
    }

    fn del_char_forward(&mut self) {
        self.dirty = true;
        self.document.del_char_forward(&self.cursor_position);
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = (self.terminal.size().height).saturating_sub(2) as usize; // -2 to account for the bar
        let offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let mut x = self.cursor_position.x;
        let mut y = self.cursor_position.y;

        let empty_row = &Row::from("");
        let mut row = self.document.row(y).unwrap_or(empty_row);

        let mut width = row.len();
        let height = self.document.len().saturating_sub(1); // -1 to account for y being 0 based
        // and len being 1 based

        match key {
            Key::Left | Key::Ctrl('b') => {
                if x > 0 { x -= 1; }
                else if y > 0 { 
                    y -= 1;
                    row = self.document.row(y).unwrap_or(empty_row);
                    width = row.len();
                    x = width;
                }
            }

            Key::Right | Key::Ctrl('f') => {
                if x < width { x = x.saturating_add(1); }
                else if y < height { 
                    y += 1;
                    x = 0;
                }
            }

            Key::Up | Key::Ctrl('p') => {
                if y > 0 { y = y.saturating_sub(1); }

                row = self.document.row(y).unwrap_or(empty_row);
                width = row.len();

                if x > width { x = width; }
            }

            Key::Down | Key::Ctrl('n') => {
                if y < height.saturating_add(1) {y = y.saturating_add(1)};

                row = self.document.row(y).unwrap_or(empty_row);
                width = row.len();

                if x > width { x = width; }
            }

            Key::Ctrl('e') => x = width,
            Key::Ctrl('a') => x = 0,

            Key::Home => {
                y = 0;
                row = self.document.row(y).unwrap_or(empty_row);
                width = row.len();
                x = width;
            }

            Key::End => {
                y = height.saturating_add(1);
                x = 0;
            }

            Key::PageUp => y = y.saturating_add(3).saturating_sub(self.terminal.size().height as usize),

            Key::PageDown => {
                if y.saturating_add(self.terminal.size().height as usize).saturating_sub(3) < self.document.len() {
                    y = y.saturating_add(self.terminal.size().height as usize).saturating_sub(3);
                } else {
                    y = self.document.len();
                }
            }

            _ => (),
        }
        self.cursor_position = Position { x, y };
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = start + width;
        let row = row.render(start, end);
        println!("{row}\r");
    }

    fn draw_status_bar(&self) {
        let mut status: String;
        let width = self.terminal.size().width as usize;
        let mut filename = String::from("[No Name]");

        if let Some(file) = &self.document.filename {
            filename = file.clone();
            filename.truncate(20);
        }
        status = format!("{}{} - {}", self.document.is_dirty().then_some("* ").unwrap_or("  ") , filename, self.document.len());
        let line_indicator = format!("{}/{}", self.cursor_position.y.saturating_add(1), self.document.len());
        let len = status.len() + line_indicator.len();

        if len < width {
            status.push_str(&" ".repeat(width-len));
        }
        status.push_str(&line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{status}\r");
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        let width = self.terminal.size().width;
        if Instant::now() - message.timestamp < Duration::new(5, 0) {
            let mut text = message.message.clone();
            text.truncate(width as usize);
            print!("{text}");
        }
    }

    fn draw_rows(&self) {
        Terminal::cursor_position(&Position::default());
        let height = self.terminal.size().height;
        for terminal_row in 0..height-2 {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
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

    fn prompt_string<C>(&mut self, prompt: &str, callback: C) -> Result<Option<String>, io::Error> 
    where
        C: Fn(&mut Self, Key, &String)
    {
        let mut ret = String::new();
        let prev_cursor_position = self.cursor_position.clone();
        self.cursor_position.y = self.terminal.size().height.saturating_sub(1) as usize;
        self.cursor_position.x = prompt.len();

        loop {
            self.status_message = StatusMessage::from(format!("{prompt}{ret}"));
            self.refresh_screen_prompt()?;

            let key = Terminal::read_key()?;
            match key {
                Key::Char('\n') => break,
                Key::Char(c) => {
                    ret.push(c);
                    self.cursor_position.x = self.cursor_position.x.saturating_add(1);
                },
                Key::Backspace => {
                    let c = ret.pop();
                    if c.is_some() {
                        self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
                    }
                },
                Key::Esc | Key::Ctrl('g') => {
                        ret.clear();
                        break;
                    }
                _ => (),
            }
            callback(self, key, &ret);
        }
        self.cursor_position = prev_cursor_position;

        self.status_message = StatusMessage::from("");
        
        if ret.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ret))
        }
    }

    fn prompt_bool(&mut self, prompt: &str) -> Result<bool, io::Error> {
		let ret: bool;
        let prev_cursor_position = self.cursor_position.clone();
        self.cursor_position.y = self.terminal.size().height.saturating_sub(1) as usize;

        loop {
            self.status_message = StatusMessage::from(format!("{prompt} y or n: "));
            self.cursor_position.x = prompt.len().saturating_add(" y or n: ".len());
			self.refresh_screen_prompt()?;

            match Terminal::read_key()? {
                Key::Char('y') => {
					ret = true;
					break;
				},
                Key::Char('n') => {
					ret = false;
					break;
				},
                Key::Esc | Key::Ctrl('g') => {
					ret = false;
					break;
				},
				_ => (),
            }
        }

        self.cursor_position = prev_cursor_position;
        self.status_message = StatusMessage::from("");
		self.refresh_screen_prompt()?;
        
		Ok(ret)
    }

    fn refresh_screen(&mut self) -> Result<(), io::Error> {
        Terminal::hide_cursor();

        let adjusted_position = Position {
            x: self.cursor_position.x.saturating_sub(self.offset.x),
            y: self.cursor_position.y.saturating_sub(self.offset.y),
        };

        Terminal::cursor_position(&adjusted_position);

        if self.should_quit {
			if self.dirty {
				if self.prompt_bool("Unsaved changes remaining. Really Quit?").unwrap() {
					Terminal::cursor_position(&Position{ x: 0, y: self.terminal.size().height.saturating_sub(1) as usize, });
					self.status_message = StatusMessage::from("");
					Terminal::clear_current_line();
					println!("Goodbye!\r");
				} else {
					self.should_quit = false;
				}
			} else {
				Terminal::cursor_position(&Position{ x: 0, y: self.terminal.size().height.saturating_sub(1) as usize, });
				self.status_message = StatusMessage::from("");
				Terminal::clear_current_line();
				println!("Goodbye!\r");
			}
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            // println!("cursor_y: {}, offset_y: {}", self.cursor_position.y, self.offset.y);
            Terminal::cursor_position(&adjusted_position);
        }
        Terminal::show_cursor();
        Terminal::flush()
    }

	fn refresh_screen_prompt(&mut self) -> Result<(), io::Error> {
        Terminal::hide_cursor();

        let adjusted_position = Position {
            x: self.cursor_position.x.saturating_sub(self.offset.x),
            y: self.cursor_position.y.saturating_sub(self.offset.y),
        };

        Terminal::cursor_position(&adjusted_position);

		self.draw_rows();
		self.draw_status_bar();
		self.draw_message_bar();
        // println!("cursor_y: {}, offset_y: {}", self.cursor_position.y, self.offset.y);
        Terminal::cursor_position(&adjusted_position);
        Terminal::show_cursor();
        Terminal::flush()
	}
}


fn die(e: &io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
