use packets::ServerPacket;
use std::time::{Duration, Instant};

pub struct Clock {
    current_tick: i64,
    current_tick_end: Instant,
    tick_duration: Duration,
    initialized: bool
}

impl Clock {
    pub fn current_tick(&self) -> i64 {
        self.current_tick
    }

    pub fn current_tick_end(&self) -> Instant {
        self.current_tick_end.clone()
    }

    pub fn handle_packet(&mut self, packet: &ServerPacket) {
        if let ServerPacket::TimeUpdate { world_age, ..} = packet {
            if self.initialized {
                if *world_age > self.current_tick {
                    self.tick_duration -= Duration::from_millis(1);
                } else if *world_age < self.current_tick && self.tick_duration.as_millis() > 1 {
                    self.tick_duration += Duration::from_millis(1);
                }
                debug!("New tick duration: {} ms", self.tick_duration.as_millis());
            } else {
                self.current_tick = *world_age;
                self.initialized = true;
            }
        }
    }

    pub fn advance(&mut self) {
        let now = Instant::now();
        while self.current_tick_end <= now {
            self.current_tick += 1;
            debug!("Tick! {}", self.current_tick());
            self.current_tick_end += self.tick_duration;
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Clock {
            current_tick: 0,
            current_tick_end: Instant::now() + Duration::from_millis(50),
            tick_duration: Duration::from_millis(50),
            initialized: false
        }
    }
}