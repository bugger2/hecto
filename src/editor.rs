use crate::Document;
use crate::Row;
use crate::terminal;
use std::io;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;
use terminal::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239); // #EFEFEF
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63); // #3F3F3F
pub const TAB_WIDTH: u32 = 4;

#[derive(Default)]
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
        StatusMessage::from(message.to_string())
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
}

impl Editor {
    pub fn default() -> Self {
        let mut initial_status = String::from("Help: Ctrl-q to exit");
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let filename = &args[1];
            let doc = Document::open(filename);
            if doc.is_ok() {
                doc.unwrap()
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
            Key::Char(c) => self.insert_char(c),
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

    fn insert_char(&mut self, c: char) {
        self.document.insert(&self.cursor_position, c);
        let x = &mut self.cursor_position.x;
        if c != '\t' {
            *x = x.saturating_add(1);
        } else {
            *x = x.saturating_add(TAB_WIDTH as usize);
        }
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
            Key::Home => y = 0,
            Key::End => y = height,
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
        status = format!("{} - {}", filename, self.document.len());
        let line_indicator = format!("{}/{}", self.cursor_position.y.saturating_add(1), self.document.len());
        let len = status.len() + line_indicator.len();

        if len < width {
            status.push_str(&" ".repeat(width-len));
        }
        status.push_str(&line_indicator);
        status.truncate(width as usize);

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

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor();

        let adjusted_position = Position {
            x: self.cursor_position.x - self.offset.x,
            y: self.cursor_position.y - self.offset.y,
        };

        Terminal::cursor_position(&adjusted_position);

        if self.should_quit {
            Terminal::cursor_position(&Position{ x: 0, y: self.terminal.size().height.saturating_sub(1) as usize, });
            println!("Goodbye!\r");
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
}


fn die(e: &io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
