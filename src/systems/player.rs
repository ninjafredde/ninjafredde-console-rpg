
use crate::systems::{
    character::Character,
    position::Position,
};
pub struct Player {
    pub character: Character,
    pub world_pos: Position,
    pub local_pos: Position,
}

impl Player {
    pub fn new(character: Character, world_x: usize, world_y: usize) -> Self {
        Self {
            character,
            world_pos: Position { x: world_x, y: world_y },
            local_pos: Position { x: 0, y: 0 },  // Will be set when entering location
        }
    }

    pub fn create_random(world_x: usize, world_y: usize) -> Self {
        Self::new(Character::create_random(), world_x, world_y)
    }

    pub fn enter_location(&mut self, spawn_x: usize, spawn_y: usize) {
        self.local_pos = Position { x: spawn_x, y: spawn_y };
    }

    pub fn exit_location(&mut self) {
        // Return to world map position
        // local_pos will be updated next time we enter a location
    }
}