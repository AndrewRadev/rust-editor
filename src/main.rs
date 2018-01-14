extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use std::fs::File;
use std::io::{self, Read, BufReader, BufRead, Write};
use std::process;

struct Buffer {

}

struct Cursor {

}

fn main() {
    let file = File::open("test.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();

    let mut stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    loop {
        render();

        handle_input(&mut stdin, &mut stdout);
    }
}

fn render() {

}

fn handle_input(stdin: &mut io::Stdin, stdout: &mut RawTerminal<io::Stdout>) {
    let c = stdin.keys().next().unwrap().unwrap();

    match c {
        Key::Char('q') => process::exit(0),
        Key::Ctrl('c') => process::exit(0),
        _              => write!(stdout, "Key pressed: {:?}", c),
    };

    stdout.flush().unwrap();
}
