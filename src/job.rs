use crate::spawn::Output;
use serde_json::json;
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
};
use ws::{Message::Text, Sender};

pub struct Socket {
    id: String,
    pub sender: Sender,
}

pub struct Job {
    id: String,
    log: RefCell<BufWriter<File>>,
    pub output: RefCell<Output>,
    pub sockets: RefCell<Vec<Socket>>,
}

impl Socket {
    pub fn new(id: String, sender: Sender) -> Socket {
        Socket { id, sender }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Job {
    pub fn new(id: String, output: Output, socket: Socket) -> Job {
        println!("logging output to file job-{}.log", id);

        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(format!("job-{}.log", id))
            .expect("couldn't open log file!");

        let log = BufWriter::new(file);

        Job {
            id,
            log: RefCell::new(log),
            output: RefCell::new(output),
            sockets: RefCell::new(vec![socket]),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn log(&self, msg: &str) -> Result<u8, &str> {
        // send to all active sockets
        let sockets = self.sockets.borrow();

        if sockets.len() == 0 {
            return Ok(0);
        }

        let mut sent_count = 0;
        for socket in sockets.iter() {
            socket
                .sender
                .send(Text(log(msg)))
                .expect("transmission error!");
            sent_count += 1;
        }

        // record to log file
        let mut log = self.log.borrow_mut();
        write!(log, "{}", msg).expect("couldn't write to log!");

        Ok(sent_count)
    }

    pub fn complete_log(&self) -> Result<u8, &str> {
        // send to all active sockets
        let sockets = self.sockets.borrow();

        if sockets.len() == 0 {
            return Ok(0);
        }

        let mut sent_count = 0;
        for socket in sockets.iter() {
            socket
                .sender
                .send(Text(
                    json!({
                        "action": {
                            "__type": "done",
                        },
                    })
                    .to_string(),
                ))
                .expect("transmission error!");
            sent_count += 1;
        }

        Ok(sent_count)
    }
}

fn log(line: &str) -> String {
    json!({
        "action": {
            "__type": "log",
            "text": line,
        },
    })
    .to_string()
}
