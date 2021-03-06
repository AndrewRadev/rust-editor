extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::clear;
use termion::cursor;

use std::fs::File;
use std::io::{self, BufReader, BufRead, Write};

#[derive(Clone)]
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

    pub fn insert(&self, c: char, row: u16, col: u16) -> Self {
        let mut lines = self.lines.clone();
        lines.get_mut(row as usize).map(|row| row.insert(col as usize, c));
        Self { lines }
    }

    pub fn delete(&self, row: u16, col: u16) -> Self {
        let mut lines = self.lines.clone();
        lines.get_mut(row as usize).map(|row| if (col as usize) < row.len() { row.remove(col as usize); });
        Self { lines }
    }

    pub fn split_line(&self, row: u16, col: u16) -> Self {
        let (lines_above, lines_below) = self.lines.split_at(row as usize);
        let (line_before, line_after) = lines_below[0].split_at(col as usize);

        let mut lines = Vec::with_capacity(self.lines.len() + 1);
        lines.extend_from_slice(lines_above);
        lines.push(line_before.to_string());
        lines.push(line_after.to_string());
        lines.extend_from_slice(&lines_below[1..]);

        Self { lines }
    }
}

#[derive(Clone)]
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
            col: Self::clamp(self.col + 1, buffer.line_length(self.row))
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
    history: Vec<(Buffer, Cursor)>,
}

impl Editor {
    pub fn new(lines: Vec<String>) -> Self {
        let stdout = io::stdout().into_raw_mode().unwrap();

        let buffer = Buffer::new(lines);
        let cursor = Cursor { row: 0, col: 0 };

        Self { stdout, buffer, cursor, history: Vec::new() }
    }

    pub fn render(&mut self) -> io::Result<()> {
        self.move_cursor(0, 0)?;
        self.buffer.render(&mut self.stdout).unwrap();
        self.reset_cursor()?;

        self.stdout.flush()
    }

    pub fn handle_input(&mut self, stdin: &mut io::Stdin) -> io::Result<bool> {
        match stdin.keys().next().unwrap().unwrap() {
            Key::Ctrl('q') => return Ok(false),
            Key::Ctrl('c') => return Ok(false),
            Key::Ctrl('z') => {
                self.restore_snapshot();
                self.clear_screen()?;
            }
            Key::Up        => self.cursor = self.cursor.up(&self.buffer),
            Key::Down      => self.cursor = self.cursor.down(&self.buffer),
            Key::Left      => self.cursor = self.cursor.left(&self.buffer),
            Key::Right     => self.cursor = self.cursor.right(&self.buffer),
            Key::Char('\n') => {
                self.save_snapshot();
                self.buffer = self.buffer.split_line(self.cursor.row, self.cursor.col);
                self.cursor = Cursor { row: self.cursor.row + 1, col: 0 };
                self.clear_screen()?;
            },
            Key::Backspace => {
                if self.cursor.col > 0 {
                    self.save_snapshot();
                    self.buffer = self.buffer.delete(self.cursor.row, self.cursor.col.saturating_sub(1));
                    self.cursor = self.cursor.left(&self.buffer);
                    self.clear_screen()?;
                }
            },
            Key::Char(c)   => {
                self.save_snapshot();
                self.buffer = self.buffer.insert(c, self.cursor.row, self.cursor.col);
                self.cursor = self.cursor.right(&self.buffer);
            },
            _ => (),
        };

        Ok(true)
    }

    pub fn save_snapshot(&mut self) {
        self.history.push((self.buffer.clone(), self.cursor.clone()));
    }

    pub fn restore_snapshot(&mut self) {
        if let Some((buffer, cursor)) = self.history.pop() {
            self.buffer = buffer;
            self.cursor = cursor;
        }
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
    editor.clear_screen().unwrap();

    loop {
        editor.render().unwrap();

        if !editor.handle_input(&mut stdin).unwrap() {
            break;
        }
    }
}
