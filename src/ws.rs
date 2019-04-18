use ::ws::{
  Message::{self, Text},
  Result,
};

pub fn connect() -> Result<()> {
  ws::listen("127.0.0.1:3000", {
    println!("listening...");

    |out| {
      println!("host connected!");

      move |msg: Message| {
        println!("received message: {}", msg);

        let mut incoming = msg.into_text().unwrap();
        incoming.push_str(" received!");
        out.send(Text(incoming))
      }
    }
  })
}
