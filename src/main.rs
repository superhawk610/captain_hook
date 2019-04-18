use captain_hook;

fn main() {
    captain_hook::connect().expect("failed to setup WS listener!");
    let stdout = captain_hook::spawn();
    for line in stdout {
        println!("LINE: {}", line);
    }
}
