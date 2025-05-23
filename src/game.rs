// src/game.rs
use crate::{world::{World}, character::Character};
use crate::location::{Location, LocationState,  Species};
use std::fmt;
use crate::location_generator::{LocationGenerator, LocationMap};

#[derive(PartialEq)]
pub enum GamePhase {
    Menu,
    PlayingWorld,
    PlayingLocation(LocationMap),
    GameOver,
}

pub struct Game {
    pub player: Character,
    pub player_pos: (usize, usize),
    pub world: World,
    pub view_radius: i32,
    pub phase: GamePhase,
    pub current_message: Option<String>,

}
impl fmt::Display for LocationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LocationState::Thriving => write!(f, "thriving"),
            LocationState::Struggling => write!(f, "struggling"),
            LocationState::Abandoned => write!(f, "abandoned"),
            LocationState::Ruins => write!(f, "ruined"),
            LocationState::Cursed => write!(f, "cursed"),
            LocationState::Sacred => write!(f, "sacred"),
            LocationState::Hidden => write!(f, "hidden"),
        }
    }
}
impl Game{
    pub fn enter_location(&mut self) {
    let current_tile = self.world.get_tile(self.player_pos.0, self.player_pos.1);
    if let Some(location) = &current_tile.location {
        let mut generator = LocationGenerator::new(
            self.world.seed,
            current_tile.terrain,
            location.clone()
        );
        
        let location_map = generator.generate();
        // Find valid spawn position
        self.player_pos = location_map.find_spawn_position();
        self.phase = GamePhase::PlayingLocation(location_map);
    }
}

    pub fn update_interaction_prompt(&mut self) {
        let current_tile = self.world.get_tile(self.player_pos.0, self.player_pos.1);
        if let Some(prompt) = self.world.get_interaction_prompt(current_tile) {
            self.current_message = Some(prompt.to_string());
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
    let current_tile = self.world.get_tile(self.player_pos.0, self.player_pos.1);
    // Clone the relevant data we need from the location
    if let Some(location) = &current_tile.location {
        let state = location.state.clone();
        let species = location.species.clone();
        let name = location.name.clone();
        
        match state {
            LocationState::Thriving | LocationState::Struggling => {
                self.set_message(format!("Entering the {} settlement of {}s", 
                    state.to_string().to_lowercase(),
                    species.to_string()));
            }
            LocationState::Abandoned | LocationState::Ruins => {
                self.set_message(format!("Exploring the {} ruins", 
                    species.to_string()));
            }
            LocationState::Sacred => {
                self.set_message(format!("You pray at the sacred site of the {}", 
                    species.to_string()));
            }
            LocationState::Cursed => {
                self.set_message("You attempt to cleanse this cursed place".to_string());
            }
            LocationState::Hidden => {
                self.set_message("You investigate the hidden location".to_string());
            }
        }
    }
}

    
    
    fn enter_settlement(&mut self, location: &Location) {
        self.set_message(format!("Entering the {} settlement of {}s", 
            location.state.to_string().to_lowercase(),
            location.species.to_string()));
    }

    fn explore_area(&mut self, location: &Location) {
        self.set_message(format!("Exploring the {} ruins", 
            location.species.to_string()));
    }

    fn pray_at_location(&mut self, location: &Location) {
        self.set_message(format!("You pray at the sacred site of the {}", 
            location.species.to_string()));
    }

    fn cleanse_location(&mut self, location: &Location) {
        self.set_message(format!("You attempt to cleanse this cursed place"));
    }

    fn investigate_location(&mut self, location: &Location) {
        self.set_message(format!("You investigate the hidden location"));
    }
    
}