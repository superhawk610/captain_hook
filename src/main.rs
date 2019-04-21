use captain_hook::{
    self,
    Hook::{Connect, Spawn},
    JobPool, Message,
};
use std::{rc::Rc, sync::Mutex};
use ws::{self, Message as WSMessage};

fn main() {
    let pool = Rc::new(Mutex::new(JobPool::new()));

    ws::listen("127.0.0.1:3000", |socket| {
        let mut p = pool.lock().unwrap();
        let job = p.register(socket);
        let id = job.get_id();

        let pool = Rc::clone(&pool);
        move |msg: WSMessage| {
            println!("received message: {}", msg);

            let incoming_str = msg.into_text().expect("can't parse message as string");
            let incoming: Message = serde_json::from_str(&incoming_str).expect("invalid JSON");

            match incoming.action {
                Spawn { arg } => {
                    let output = captain_hook::spawn(&arg);
                    let pool = pool.lock().unwrap();
                    let job = pool.get(id).unwrap();
                    job.set_output(output);
                    job.run();
                }
                Connect { id } => {
                    let _job = &pool.lock().unwrap().get(id).unwrap();
                    // TODO: connect to running job output
                }
                _ => panic!("unrecognized hook"),
            }

            Ok(())
        }
    })
    .expect("failed to setup WS listener!");
}
