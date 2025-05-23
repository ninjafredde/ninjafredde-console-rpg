mod character;
use character::{Character};
mod world;
use world::World;
use world::{MAP_WIDTH, MAP_HEIGHT, FeatureType};
mod game;
use game::{Game, GamePhase};
mod input;
use input::handle_world_input;
mod location;
use location::Location;
mod render;
use render::{render_game, shutdown_terminal, init_terminal, GameTerminal};

use rand::{Rng};




use std::{io, time::Duration, time::Instant};

fn main() {

    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal: GameTerminal = init_terminal()?;

    // Construct initial game state
    let mut game = Game {
        player: Character::create_random(), // or Character::create_human("Name".to_string()),
        player_pos: (MAP_WIDTH / 2, MAP_HEIGHT / 2),
        world: world::generate_world_map(12345),
        view_radius: 10,
        phase: GamePhase::PlayingWorld,
        current_message: None,
    };

    // place the player in a town
    let start_pos = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
    if let Some(pos) = game.world.find_nearest_species(start_pos, game.player.species) {
        println!("Nearest town is at {:?}", pos);
        game.player_pos = pos;
    } else {
        println!("No towns found!");
}
    // Game loop
    loop {
        match game.phase {
            GamePhase::Menu => {
                // TODO: render menu, handle menu input
            }
            GamePhase::PlayingWorld => {
                handle_world_input(&mut game);
                game.world.update(game.player_pos);
                render_game(&mut terminal, &game);
            }
            GamePhase::GameOver => {
                shutdown_terminal(&mut terminal);
                std::process::exit(0);
            }
        }
    }
    // Cleanup and shutdown
    shutdown_terminal(&mut terminal);
}