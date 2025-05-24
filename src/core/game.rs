use crate::systems::world::World;
use crate::systems::player::Player;
use crate::systems::location::{Location, LocationState, Species};
use crate::generators::location_generator::{LocationGenerator, LocationMap};

#[derive(PartialEq)]
pub enum GamePhase {
    Menu,
    PlayingWorld,
    PlayingLocation(LocationMap),
    GameOver,
}

pub struct Game {
    pub player: Player,
    pub world: World,
    pub view_radius: i32,
    pub phase: GamePhase,
    pub current_message: Option<String>,
}

impl Game {
    // Location-related methods
    pub fn enter_location(&mut self) {
        let current_tile = self.world.get_tile(&self.player.world_pos);
        if let Some(location) = &current_tile.location {
            let mut generator = LocationGenerator::new(
                self.world.seed,
                current_tile.terrain,
                location.clone()
            );
            
            let location_map = generator.generate();
            let spawn_pos = location_map.find_spawn_position();
            self.player.enter_location(spawn_pos.x, spawn_pos.y);
            self.phase = GamePhase::PlayingLocation(location_map);
        }
    }

    pub fn exit_location(&mut self) {
        self.player.exit_location();
        self.phase = GamePhase::PlayingWorld;
    }

    // Message handling methods
    pub fn update_interaction_prompt(&mut self) {
        let current_tile = self.world.get_tile(&self.player.world_pos);
        if let Some(prompt) = self.world.get_interaction_prompt(current_tile) {
            self.current_message = Some(prompt);
        } else {
            self.current_message = None;
        }
    }

    pub fn set_message(&mut self, message: String) {
        self.current_message = Some(message);
    }

    pub fn clear_message(&mut self) {
        self.current_message = None;
    }

    pub fn handle_interaction(&mut self) {
        let current_tile = self.world.get_tile(&self.player.world_pos);
        if let Some(location) = &current_tile.location {
            let state = location.state;
            let species = location.species;
            
            let message = match state {
                LocationState::Thriving | LocationState::Struggling => {
                    format!("Entering the {} settlement of {}s", 
                        state.to_string().to_lowercase(),
                        species.to_string())
                }
                LocationState::Abandoned | LocationState::Ruins => {
                    format!("Exploring the {} ruins", species.to_string())
                }
                LocationState::Sacred => {
                    format!("You pray at the sacred site of the {}", species.to_string())
                }
                LocationState::Cursed => {
                    "You attempt to cleanse this cursed place".to_string()
                }
                LocationState::Hidden => {
                    "You investigate the hidden location".to_string()
                }
            };
            self.set_message(message);
        }
    }
}