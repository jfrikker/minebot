use minebot::{MinebotClient, Result};
use minebot::events::{EventMatcher, EventMatchers};
use stderrlog;

fn main() {
    stderrlog::new()
            .verbosity(5)
            .init()
            .unwrap();
    let mut client = MinebotClient::connect("localhost".to_owned(), 25565, "bilbo".to_owned()).unwrap();
    client.say("Hey! I'm a bot!").unwrap();
    ping_position(&mut client).unwrap();
    println!("Health: {}", client.health());

    let mut matchers = EventMatchers::default();
    matchers.listen(EventMatcher::ChatMessage);

    let event = client.poll_until_event(&matchers).unwrap();
    println!("{:?}", event);
}

fn ping_position(client: &mut MinebotClient) -> Result<()> {
    let position = client.my_position();
    client.say(format!("My position is: ({}, {}, {})", position.x, position.y, position.z))
}