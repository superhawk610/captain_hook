use captain_hook::{
    self,
    Hook::{Connect, Spawn},
    Message, Pool,
};
use std::rc::Rc;
use ws::{self, Message as WSMessage};

fn main() {
    let pool = Rc::new(Pool::new());

    ws::listen("127.0.0.1:3000", |socket| {
        println!("received connection");

        let socket_id = pool.register(socket);

        let pool = Rc::clone(&pool);
        move |msg: WSMessage| {
            println!("received message: {}", msg);

            let incoming_str = msg.into_text().expect("can't parse message as string");
            let incoming: Message = serde_json::from_str(&incoming_str).expect("invalid JSON");

            match incoming.action {
                Spawn { arg } => {
                    println!("spawning {}", arg);
                    pool.spawn(&arg, &socket_id);
                }
                Connect { id } => {
                    println!("connecting to {}", id);
                    pool.assign_socket_to_job(&socket_id, &id)
                        .expect(&format!("couldn't connect to job {}", id));
                }
                _ => panic!("unrecognized hook"),
            }

            Ok(())
        }
    })
    .expect("failed to setup WS listener!");
}
