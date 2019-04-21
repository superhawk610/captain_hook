use crate::message::{Job, Output};
use ::serde_json::json;
use std::{
    fs,
    io::Read,
    process::{Command, Stdio},
};

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
                        chars.push('\n' as u8);
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

impl Job {
    pub fn run(&self) {
        let mut output = self.output.borrow_mut();
        if let Some(output) = &mut *output {
            println!("running job {}", self.get_id());

            // pipe process's output to the socket
            for line in &mut *output {
                self.send(&log(&line)).unwrap();
            }

            // notify the socket that it's complete
            self.send(
                &json!({
                    "action": "done",
                })
                .to_string(),
            )
            .unwrap();
        } else {
            println!("no process configured for job!");
        }
    }
}

pub fn spawn(arg: &str) -> Output {
    let child = Command::new("/usr/bin/git")
        .arg(arg)
        .current_dir(fs::canonicalize(std::env::current_dir().unwrap()).unwrap())
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

fn log(line: &str) -> String {
    json!({
        "action": "log",
        "content": line,
    })
    .to_string()
}
