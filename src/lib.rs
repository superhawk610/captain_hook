mod ws;

use std::{
    io::{Bytes, Read},
    process::{ChildStdout, Command, Stdio},
};

pub use crate::ws::connect;

pub struct Output {
    stdout: Bytes<ChildStdout>,
    closed: bool,
}

impl Iterator for Output {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // closed outputs contain empty iterators
        if self.closed {
            return None;
        }

        let mut chars = vec![];
        let mut closed = true;

        for byte in &mut self.stdout {
            if let Ok(b) = byte {
                match b as char {
                    '\r' => (),
                    '\n' => {
                        closed = false;
                        break;
                    }
                    _ => chars.push(b),
                }
            }
        }

        // if iterator is empty, mark this output as closed
        if closed {
            self.closed = true;
        }

        Some(String::from_utf8_lossy(&chars).to_string())
    }
}

pub fn spawn() -> Output {
    let child = Command::new("/usr/bin/git")
        .arg("status")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute `git status`");

    Output {
        stdout: child
            .stdout
            .expect("failed to attach to child stdout")
            .bytes(),
        closed: false,
    }
}
