#[macro_use] extern crate cpython;
#[macro_use] extern crate log;

use cpython::*;
use minebot;
use minebot::events as events;
use minebot::events::{EventMatcher};
use minebot::gamestate as gamestate;
use minebot::geom::{Distance, Position};
use std::cell::RefCell;

py_class!(class MinebotClient |py| {
    data client: RefCell<minebot::MinebotClient>;

    def listen_for(&self, matchers: &EventMatchers) -> PyResult<Event> {
        let m: &events::EventMatchers = &matchers.matchers(py).borrow();
        let event = self.client(py).borrow_mut().poll_until_event(m).unwrap();
        Event::create_instance(py, event)
    }

    def get_health(&self) -> PyResult<f32> {
        let health = self.client(py).borrow().get_health();
        trace!("Health is {}", health);
        Ok(health)
    }
    
    def get_food(&self) -> PyResult<f32> {
        let food = self.client(py).borrow().get_food();
        trace!("Food is {}", food);
        Ok(food)
    }
    
    def get_my_position(&self) -> PyResult<(Distance, Distance, Distance)> {
        let client = self.client(py).borrow();
        let position = client.get_my_position();
        let position_tup = (position.x(), position.y(), position.z());
        trace!("My position is {:?}", position_tup);
        Ok(position_tup)
    }

    def say(&self, message: String) -> PyResult<Option<i32>> {
        self.client(py).borrow_mut().say(message).unwrap();
        Ok(None)
    }

    def get_block_state_at(&self, position: (f64, f64, f64)) -> PyResult<Option<BlockState>> {
        let pos = Position::new(position.0, position.1, position.2);
        self.client(py)
            .borrow()
            .get_block_state_at(&pos)
            .map(|bs| BlockState::create_instance(py, bs))
            .transpose()
    }
});

py_class!(class EventMatchers |py| {
    data matchers: RefCell<events::EventMatchers>;

    def __new__(_cls) -> PyResult<EventMatchers> {
        EventMatchers::create_instance(py, RefCell::default())
    }

    def listen_chat(&self) -> PyResult<Option<i32>> {
        self.matchers(py).borrow_mut().listen(EventMatcher::ChatMessage);
        Ok(None)
    }

    def listen_health(&self) -> PyResult<Option<i32>> {
        self.matchers(py).borrow_mut().listen(EventMatcher::HealthChanged);
        Ok(None)
    }
});

py_class!(class Event |py| {
    data event: events::Event;

    def is_chat(&self) -> PyResult<bool> {
        Ok(match self.event(py) {
            events::Event::ChatMessage { .. } => true,
            _ => false
        })
    }

    def chat_player(&self) -> PyResult<Option<String>> {
        Ok(match self.event(py) {
            events::Event::ChatMessage { player, .. } => Some(player.clone()),
            _ => None
        })
    }

    def chat_message(&self) -> PyResult<Option<String>> {
        Ok(match self.event(py) {
            events::Event::ChatMessage { message, .. } => Some(message.clone()),
            _ => None
        })
    }

    def is_health(&self) -> PyResult<bool> {
        Ok(match self.event(py) {
            events::Event::HealthChanged { .. } => true,
            _ => false
        })
    }

    def health_old(&self) -> PyResult<Option<f32>> {
        Ok(match self.event(py) {
            events::Event::HealthChanged { old, .. } => Some(*old),
            _ => None
        })
    }

    def health_new(&self) -> PyResult<Option<f32>> {
        Ok(match self.event(py) {
            events::Event::HealthChanged { new, .. } => Some(*new),
            _ => None
        })
    }
});

py_class!(class BlockState |py| {
    data id: gamestate::BlockState;

    def get_id(&self) -> PyResult<u16> {
        Ok(self.id(py).get_id())
    }

    def get_meta(&self) -> PyResult<u8> {
        Ok(self.id(py).get_meta())
    }
});

py_module_initializer!(libminebot, initlibminebot, PyInit_libminebot, |py, m| {
    stderrlog::new()
            .verbosity(3)
            .init()
            .unwrap();

    m.add(py, "__doc__", "This module is implemented in Rust.")?;
    m.add(py, "connect", py_fn!(py, connect(host: String, port: u16, username: String)))?;
    m.add(py, "connect_local", py_fn!(py, connect_local(username: String)))?;
    m.add_class::<EventMatchers>(py)?;
    Ok(())
});

fn connect(py: Python, host: String, port: u16, username: String) -> PyResult<MinebotClient> {
    println!("Hello!");
    let client = minebot::MinebotClient::connect(host, port, username).unwrap();
    MinebotClient::create_instance(py, RefCell::new(client))
}

fn connect_local(py: Python, username: String) -> PyResult<MinebotClient> {
    connect(py, "localhost".to_owned(), 25565, username)
}