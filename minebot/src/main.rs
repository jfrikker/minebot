use minebot::MinebotClient;
use stderrlog;

fn main() {
    stderrlog::new()
            .verbosity(5)
            .init()
            .unwrap();
    let mut client = MinebotClient::connect("localhost".to_owned(), 25565, "bilbo".to_owned()).unwrap();

    //loop {
    //    client.poll().unwrap();
    //}
}