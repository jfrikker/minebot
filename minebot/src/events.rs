use crate::gamestate::GameState;
use json::JsonValue;
use packets::{PlayerListPacket, ServerPacket};

pub trait EventMatcher {
    type Event;

    fn match_packet(&self, packet: &ServerPacket, prestate: &GameState) -> Option<Self::Event>;
    fn match_tick(&self, tick: i64) -> Option<Self::Event>;
}

#[derive(Clone)]
pub enum StandardEventMatcher {
    ChatMessage,
    HealthChanged,
    PlayerListChanged,
    TickReached(i64)
}

impl EventMatcher for StandardEventMatcher {
    type Event = StandardEvent;

    fn match_packet(&self, packet: &ServerPacket, gamestate: &GameState) -> Option<StandardEvent> {
        match (self, packet) {
            (StandardEventMatcher::ChatMessage, ServerPacket::ChatMessage { json, .. }) => {
                if let Some((player, message)) = parse_chat(json) {
                    if player != gamestate.my_username() {
                        Some(StandardEvent::ChatMessage {
                            player,
                            message
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            (StandardEventMatcher::HealthChanged, ServerPacket::UpdateHealth { health, ..}) => {
                let health = health / 2.0;
                if health != gamestate.health() {
                    Some(StandardEvent::HealthChanged{
                        new: health,
                        old: gamestate.health()
                    })
                } else {
                    None
                }
            }

            (StandardEventMatcher::PlayerListChanged, ServerPacket::PlayerList { packet: PlayerListPacket::AddPlayers { ref players } }) => {
                let added = players.into_iter()
                    .map(|player| player.name.to_string())
                    .collect();
                Some(StandardEvent::PlayersJoined {
                    usernames: added
                })
            }

            (StandardEventMatcher::PlayerListChanged, ServerPacket::PlayerList { packet: PlayerListPacket::RemovePlayers { players } }) => {
                let removed = players.into_iter()
                    .filter_map(|player| gamestate.player_name(&player.uuid).map(|p| p.to_string()))
                    .collect();
                Some(StandardEvent::PlayersLeft {
                    usernames: removed
                })
            }

            _ => None
        }
    }

    fn match_tick(&self, tick: i64) -> Option<StandardEvent> {
        match *self {
            StandardEventMatcher::TickReached(target_tick) => {
                if tick >= target_tick {
                    Some(StandardEvent::TickReached {
                        tick: target_tick
                    })
                } else {
                    None
                }
            },
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum StandardEvent {
    ChatMessage {
        player: String,
        message: String
    },
    HealthChanged {
        new: f32,
        old: f32
    },
    PlayersJoined {
        usernames: Vec<String>
    },
    PlayersLeft {
        usernames: Vec<String>
    },
    TickReached {
        tick: i64
    }
}

impl <M: EventMatcher> EventMatcher for Vec<M> {
    type Event = M::Event;

    fn match_packet(&self, packet: &ServerPacket, gamestate: &GameState) -> Option<Self::Event> {
        self.iter()
            .filter_map(|m| m.match_packet(packet, gamestate))
            .next()
    }

    fn match_tick(&self, tick: i64) -> Option<Self::Event> {
        self.iter()
            .filter_map(|m| m.match_tick(tick))
            .next()
    }
}

fn parse_chat(chat: &JsonValue) -> Option<(String, String)> {
    let player: String = chat["with"][0]["text"].as_str()?.to_owned();
    let mut message = chat["with"][1]["extra"].members()
        .flat_map(|v| v["text"].as_str().map(|s| s.to_owned()))
        .collect::<Vec<String>>()
        .join("");
    if message.is_empty() {
        message = chat["with"][1].as_str()?.to_owned()
    }
    Some((player, message))
}