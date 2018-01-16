extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::clear;
use termion::cursor;

use std::fs::File;
use std::io::{self, BufReader, BufRead, Write};

struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new(lines: Vec<String>) -> Self {
        Buffer { lines }
    }

    pub fn render(&mut self, stdout: &mut RawTerminal<io::Stdout>) -> io::Result<()> {
        for line in self.lines.iter() {
            write!(stdout, "{}\r\n", line)?;
        }
        Ok(())
    }

    pub fn line_count(&self) -> u16 {
        self.lines.len() as u16
    }

    pub fn line_length(&self, row: u16) -> u16 {
        self.lines.get(row as usize).map(String::len).unwrap_or(0) as u16
    }
}

struct Cursor {
    row: u16,
    col: u16,
}

impl Cursor {
    pub fn up(&self, buffer: &Buffer) -> Self {
        let new_row = self.row.saturating_sub(1);

        Self {
            row: new_row,
            col: Self::clamp(self.col, buffer.line_length(new_row).saturating_sub(1)),
        }
    }

    pub fn down(&self, buffer: &Buffer) -> Self {
        let new_row = Self::clamp(self.row + 1, buffer.line_count().saturating_sub(1));

        Self {
            row: new_row,
            col: Self::clamp(self.col, buffer.line_length(new_row).saturating_sub(1)),
        }
    }

    pub fn left(&self, _buffer: &Buffer) -> Self {
        Self {
            row: self.row,
            col: self.col.saturating_sub(1),
        }
    }

    pub fn right(&self, buffer: &Buffer) -> Self {
        Self {
            row: self.row,
            col: Self::clamp(self.col + 1, buffer.line_length(self.row).saturating_sub(1))
        }
    }

    fn clamp(n: u16, limit: u16) -> u16 {
        if n > limit {
            limit
        } else {
            n
        }
    }
}

struct Editor {
    stdout: RawTerminal<io::Stdout>,
    buffer: Buffer,
    cursor: Cursor,
}

impl Editor {
    pub fn new(lines: Vec<String>) -> Self {
        let stdout = io::stdout().into_raw_mode().unwrap();

        let buffer = Buffer::new(lines);
        let cursor = Cursor { row: 0, col: 0 };

        Self { stdout, buffer, cursor }
    }

    pub fn render(&mut self) -> io::Result<()> {
        self.clear_screen()?;

        self.move_cursor(0, 0)?;
        self.buffer.render(&mut self.stdout).unwrap();
        self.reset_cursor()?;

        self.stdout.flush()
    }

    pub fn handle_input(&mut self, stdin: &mut io::Stdin) -> io::Result<bool> {
        let c = stdin.keys().next().unwrap().unwrap();

        match c {
            Key::Ctrl('q') => return Ok(false),
            Key::Ctrl('c') => return Ok(false),
            Key::Up        => self.cursor = self.cursor.up(&self.buffer),
            Key::Down      => self.cursor = self.cursor.down(&self.buffer),
            Key::Left      => self.cursor = self.cursor.left(&self.buffer),
            Key::Right     => self.cursor = self.cursor.right(&self.buffer),
            _              => write!(self.stdout, "Key pressed: {:?}\r\n", c)?,
        };

        Ok(true)
    }

    fn clear_screen(&mut self) -> io::Result<()> {
        write!(self.stdout, "{}", clear::All)
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        write!(self.stdout, "{}", cursor::Goto(x + 1, y + 1))
    }

    fn reset_cursor(&mut self) -> io::Result<()> {
        write!(self.stdout, "{}", cursor::Goto(self.cursor.col + 1, self.cursor.row + 1))
    }
}

fn main() {
    let file = File::open("test.txt").expect("Couldn't open file!");
    let reader = BufReader::new(file);
    let lines = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();
    let mut stdin = io::stdin();

    let mut editor = Editor::new(lines);

    loop {
        editor.render().unwrap();

        if !editor.handle_input(&mut stdin).unwrap() {
            break;
        }
    }
}
