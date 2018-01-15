extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

extern crate editor;
use editor::ansi;

use std::fs::File;
use std::io::{self, Read, BufReader, BufRead, Write};

struct Buffer {
    stdout: RawTerminal<io::Stdout>,
    lines: Vec<String>,
}

impl Buffer {
    fn new(lines: Vec<String>, stdout: RawTerminal<io::Stdout>) -> Self {
        Buffer { lines, stdout }
    }

    fn render(&mut self) -> io::Result<()> {
        for line in self.lines.iter() {
            write!(self.stdout, "{}\r\n", line)?;
        }
        Ok(())
    }
}

struct Cursor {

}

fn main() {
    let file = File::open("test.txt").expect("Couldn't open file!");
    let reader = BufReader::new(file);
    let lines = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();

    let mut stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    let mut buffer = Buffer::new(lines, io::stdout().into_raw_mode().unwrap());

    loop {
        render(&mut stdout).unwrap();
        buffer.render().unwrap();

        if !handle_input(&mut stdin, &mut stdout) {
            break;
        }
    }
}

fn render(mut stdout: &mut RawTerminal<io::Stdout>) -> io::Result<()> {
    ansi::clear_screen(&mut stdout)?;
    ansi::move_cursor(&mut stdout, 0, 0)?;

    stdout.flush()
}

fn handle_input(stdin: &mut io::Stdin, stdout: &mut RawTerminal<io::Stdout>) -> bool {
    let c = stdin.keys().next().unwrap().unwrap();

    match c {
        Key::Ctrl('q') => return false,
        _              => write!(stdout, "Key pressed: {:?}\r\n", c),
    };

    stdout.flush().unwrap();
    true
}
