#[macro_use] extern crate log;

use cpython::*;
use minebot;
use minebot::blocks as blocks;
use minebot::events::{StandardEvent, StandardEventMatcher};
use minebot::geom::{BlockPosition, Position, to_block_position};
use std::cell::RefCell;

py_class!(class MinebotClient |py| {
    data client: RefCell<minebot::MinebotClient>;

    def listen_for(&self, matchers: &EventMatcher) -> PyResult<Event> {
        let m: &Vec<StandardEventMatcher> = &matchers.matchers(py).borrow();
        let event = self.client(py).borrow_mut().poll_until_event(m).unwrap();
        Event::create_instance(py, event)
    }

    def get_health(&self) -> PyResult<f32> {
        let health = self.client(py).borrow().health();
        trace!("Health is {}", health);
        Ok(health)
    }
    
    def get_food(&self) -> PyResult<f32> {
        let food = self.client(py).borrow().food();
        trace!("Food is {}", food);
        Ok(food)
    }
    
    def get_my_position(&self) -> PyResult<(f64, f64, f64)> {
        let client = self.client(py).borrow();
        let position = client.my_position();
        let position_tup = (position.x, position.y, position.z);
        trace!("My position is {:?}", position_tup);
        Ok(position_tup)
    }

    def get_player_names(&self) -> PyResult<Vec<String>> {
        Ok(self.client(py).borrow().player_names().into_iter()
            .map(|p| p.to_string())
            .collect())
    }

    def say(&self, message: String) -> PyResult<Option<i32>> {
        self.client(py).borrow_mut().say(message).unwrap();
        Ok(None)
    }

    def get_block_state_at(&self, position: (f64, f64, f64)) -> PyResult<Option<BlockState>> {
        let pos = read_block_position(position);
        self.client(py)
            .borrow()
            .block_state_at(pos)
            .map(|bs| BlockState::create_instance(py, bs))
            .transpose()
    }

    def find_block_ids_within(&self, block_id: u16, position: (f64, f64, f64), distance: i32) -> PyResult<Vec<(f64, f64, f64)>> {
        let pos = read_block_position(position);
        Ok(self.client(py).borrow().find_block_ids_within(block_id, pos, distance).into_iter()
            .map(|pos| (pos.x as f64, pos.y as f64, pos.z as f64))
            .collect())
    }

    def find_path_to(&self, start: (f64, f64, f64), end: (f64, f64, f64)) -> PyResult<Option<Vec<(f64, f64, f64)>>> {
        let start_pos = read_block_position(start);
        let end_pos = read_block_position(end);
        Ok(self.client(py).borrow().find_path_to(start_pos, end_pos).map(|r| r.into_iter()
            .map(|pos| (pos.x as f64, pos.y as f64, pos.z as f64))
            .collect()))
    }

    def current_tick(&self) -> PyResult<i64> {
        Ok(self.client(py).borrow().current_tick())
    }

    def teleport_to(&self, position: (f64, f64, f64)) -> PyResult<Option<i32>> {
        self.client(py).borrow_mut().teleport_to(read_position(position)).unwrap();
        Ok(None)
    }

    def set_my_yaw(&self, angle: f32) -> PyResult<Option<i32>> {
        self.client(py).borrow_mut().set_my_yaw(angle).unwrap();
        Ok(None)
    }

    def enable_move(&self, flag: bool) -> PyResult<Option<i32>> {
        self.client(py).borrow_mut().r#move(flag);
        Ok(None)
    }
});

fn read_position(position: (f64, f64, f64)) -> Position {
    Position::new(position.0, position.1, position.2)
}

fn read_block_position(position: (f64, f64, f64)) -> BlockPosition {
    to_block_position(read_position(position))
}

py_class!(class EventMatcher |py| {
    data matchers: RefCell<Vec<StandardEventMatcher>>;

    def __new__(_cls, other: Option<&EventMatcher> = None) -> PyResult<EventMatcher> {
        let result = EventMatcher::create_instance(py, RefCell::default())?;
        if let Some(other_matcher) = other {
            result.matchers(py).borrow_mut().extend(other_matcher.matchers(py).borrow().iter().cloned());
        }
        Ok(result)
    }

    def listen_chat(&self) -> PyResult<Option<i32>> {
        self.matchers(py).borrow_mut().push(StandardEventMatcher::ChatMessage);
        Ok(None)
    }

    def listen_health(&self) -> PyResult<Option<i32>> {
        self.matchers(py).borrow_mut().push(StandardEventMatcher::HealthChanged);
        Ok(None)
    }

    def listen_player_list(&self) -> PyResult<Option<i32>> {
        self.matchers(py).borrow_mut().push(StandardEventMatcher::PlayerListChanged);
        Ok(None)
    }

    def listen_tick(&self, target_tick: i64) -> PyResult<Option<i32>> {
        self.matchers(py).borrow_mut().push(StandardEventMatcher::TickReached(target_tick));
        Ok(None)
    }
});

py_class!(class Event |py| {
    data event: StandardEvent;

    def is_chat(&self) -> PyResult<bool> {
        Ok(match self.event(py) {
            StandardEvent::ChatMessage { .. } => true,
            _ => false
        })
    }

    def chat_player(&self) -> PyResult<Option<String>> {
        Ok(match self.event(py) {
            StandardEvent::ChatMessage { player, .. } => Some(player.clone()),
            _ => None
        })
    }

    def chat_message(&self) -> PyResult<Option<String>> {
        Ok(match self.event(py) {
            StandardEvent::ChatMessage { message, .. } => Some(message.clone()),
            _ => None
        })
    }

    def is_health(&self) -> PyResult<bool> {
        Ok(match self.event(py) {
            StandardEvent::HealthChanged { .. } => true,
            _ => false
        })
    }

    def health_old(&self) -> PyResult<Option<f32>> {
        Ok(match self.event(py) {
            StandardEvent::HealthChanged { old, .. } => Some(*old),
            _ => None
        })
    }

    def health_new(&self) -> PyResult<Option<f32>> {
        Ok(match self.event(py) {
            StandardEvent::HealthChanged { new, .. } => Some(*new),
            _ => None
        })
    }

    def is_player_list(&self) -> PyResult<bool> {
        Ok(match self.event(py) {
            StandardEvent::PlayersJoined { .. } => true,
            StandardEvent::PlayersLeft { .. } => true,
            _ => false
        })
    }

    def players_joined(&self) -> PyResult<Vec<String>> {
        Ok(match self.event(py) {
            StandardEvent::PlayersJoined { usernames } => usernames.clone(),
            _ => Vec::new()
        })
    }

    def players_left(&self) -> PyResult<Vec<String>> {
        Ok(match self.event(py) {
            StandardEvent::PlayersLeft { usernames } => usernames.clone(),
            _ => Vec::new()
        })
    }

    def is_tick_reached(&self) -> PyResult<bool> {
        Ok(match self.event(py) {
            StandardEvent::TickReached { .. } => true,
            _ => false
        })
    }

    def tick_reached(&self) -> PyResult<Option<i64>> {
        Ok(match self.event(py) {
            StandardEvent::TickReached { tick } => Some(*tick),
            _ => None
        })
    }
});

py_class!(class BlockState |py| {
    data id: blocks::BlockState;

    def get_id(&self) -> PyResult<u16> {
        Ok(self.id(py).id())
    }

    def get_meta(&self) -> PyResult<u8> {
        Ok(self.id(py).meta())
    }
});

py_module_initializer!(libminebot, initlibminebot, PyInit_libminebot, |py, m| {
    stderrlog::new()
            .verbosity(4)
            .init()
            .unwrap();

    m.add(py, "__doc__", "This module is implemented in Rust.")?;
    m.add(py, "connect", py_fn!(py, connect(host: String, port: u16, username: String)))?;
    m.add(py, "connect_local", py_fn!(py, connect_local(username: String)))?;
    m.add_class::<EventMatcher>(py)?;
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
