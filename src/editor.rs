
use std::io::{self, stdout};
use termion::event::Key;
use termoin::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Editor {}

impl Editor {
    pub fn run(&self) {
        let _stdout = stdout().into_raw_mode().unwrap();

        for key in io::stdin().keys() {
            match key {
                Ok(key) => handle_key(key),
                Err(err) => die(err),
            }
        }
    }
    pub fn default() -> Self {
        Self{}
    }
}

fn handle_key(k) {
    match k {
        Key::Char(c) => {
            if c.is_control() {
                println!("{:?}\r", c as u8);
            } else {
                println!("{:?} ({})\r", c as u8, c)
            }
        }
        Key::Ctrl('q') => break,
        _ => println!() 
    }
}

fn die(e: io::Error) {
    panic!(e);
}
