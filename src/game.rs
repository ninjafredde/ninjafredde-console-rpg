// src/game.rs
use crate::{world::World, character::Character};
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum GamePhase {
    Menu,
    PlayingWorld,
    GameOver,
}

pub struct Game {
    pub player: Character,
    pub player_pos: (usize, usize),
    pub world: World,
    pub view_radius: i32,
    pub phase: GamePhase,
    pub last_input_time: Instant,
    pub input_cooldown: Duration,
}
