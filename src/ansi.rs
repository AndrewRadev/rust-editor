use std::io::{self, Write};
use termion::raw::RawTerminal;
use termion::clear;
use termion::cursor;

pub fn clear_screen(stdout: &mut RawTerminal<io::Stdout>) -> io::Result<()> {
    write!(stdout, "{}", clear::All)
}

pub fn move_cursor(stdout: &mut RawTerminal<io::Stdout>, x: u16, y: u16) -> io::Result<()> {
    write!(stdout, "{}", cursor::Goto(x + 1, y + 1))
}
