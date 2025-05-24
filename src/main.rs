mod core;
mod systems;
mod render;
mod generators;
mod prelude;

use prelude::*;

use core::{game::Game, input::handle_input};
use render::tui_render::{render_game, init_terminal, shutdown_terminal, GameTerminal};
use systems::world::MAP_WIDTH;
fn main() {

    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    
}
fn run() -> Result<()> {
    let mut terminal: GameTerminal = init_terminal()?;

    let mut player = Player::create_random(MAP_WIDTH / 2, MAP_HEIGHT / 2);
    player.character = Character::create_random(); // or Character::create_human("Name".to_string());

    // Construct initial game state
    let mut game = Game {
        player: player,
        world: World::new(MAP_WIDTH,12345),
        view_radius: 10,
        phase: GamePhase::PlayingWorld,
        current_message: None,
    };

    // Place the player in a town using Position
    let start_pos = Position {
        x: MAP_WIDTH / 2,
        y: MAP_HEIGHT / 2,
    };

    if let Some(pos) = game.world.find_nearest_species(&start_pos, game.player.character.species) {
        println!("Nearest town is at {:?}", pos);
        game.player.world_pos = pos;
    } else {
        println!("No towns found!");
    }

    // Game loop
    loop {
        // First render the current game state
        render_game(&mut terminal, &game)?;

        // Then handle the current phase
        match game.phase {
            GamePhase::Menu => {
                // TODO: render menu, handle menu input
            }
            GamePhase::PlayingWorld | GamePhase::PlayingLocation(_) => {
                handle_input(&mut game)?;
                game.world.update(&game.player.world_pos);
            }
            GamePhase::GameOver => {
                break;
            }
        }
    }
    // Cleanup and shutdown
    shutdown_terminal(&mut terminal)?;  // Add ? operator here
    Ok(())  // Add explicit Ok return
}