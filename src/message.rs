use ::serde::{Deserialize, Serialize};
use ::ws::{Message::Text, Sender};
use std::{cell::RefCell, io::Bytes, process::ChildStdout};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub action: Hook,
    #[serde(skip_deserializing)]
    pub identifier: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum Hook {
    #[serde(rename = "spawn")]
    Spawn { arg: String },
    #[serde(rename = "connect")]
    Connect { id: u32 },
    #[serde(rename = "confirm")]
    Confirm,
    #[serde(rename = "log")]
    Log,
    #[serde(rename = "done")]
    Done,
}

pub struct Output {
    pub stdout: Bytes<ChildStdout>,
    pub closed: bool,
}

pub struct JobPool {
    id: u32,
    jobs: Vec<Job>,
}

pub struct Job {
    id: u32,
    pub output: RefCell<Option<Output>>,
    pub sockets: Vec<Sender>,
}

impl JobPool {
    pub fn new() -> JobPool {
        JobPool {
            id: 0,
            jobs: Vec::new(),
        }
    }

    pub fn register(&mut self, socket: Sender) -> &Job {
        self.id += 1;

        self.jobs.push(Job {
            id: self.id,
            output: RefCell::new(None),
            sockets: vec![socket],
        });

        &self.jobs.last().unwrap()
    }

    pub fn get(&self, id: u32) -> Result<&Job, String> {
        for job in &self.jobs {
            if job.id == id {
                return Ok(job);
            }
        }

        Err(format!("no job matching id {} found", id))
    }

    pub fn remove(&mut self, remove: &Job) -> Result<(), String> {
        for (idx, job) in self.jobs.iter().enumerate() {
            if job.id == remove.id {
                self.jobs.remove(idx);
                return Ok(());
            }
        }

        Err(format!("no job matching id {} found to remove", remove.id))
    }
}

impl Job {
    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn send(&self, msg: &str) -> Result<u8, &str> {
        if self.sockets.len() == 0 {
            return Err("no open sockets to send message");
        }

        let mut sent_count = 0;
        for socket in &self.sockets {
            socket
                .send(Text(msg.to_string()))
                .expect("transmission error!");
            sent_count += 1;
        }

        Ok(sent_count)
    }

    pub fn set_output(&self, output: Output) {
        self.output.replace(Some(output));
    }
}
