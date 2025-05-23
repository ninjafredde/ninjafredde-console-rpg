mod character;
use character::{Character};
mod world;
use world::World;
mod world_generator;
use world::{MAP_WIDTH, MAP_HEIGHT, FeatureType};
mod game;
use game::{Game, GamePhase};
mod input;
use input::handle_input;
mod location;
mod location_generator;
mod render;
use render::{render_game, shutdown_terminal, init_terminal, GameTerminal};

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
        world: World::new(MAP_WIDTH,12345),
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
        // First render the current game state
        render_game(&mut terminal, &game)?;

        // Then handle the current phase
        match game.phase {
            GamePhase::Menu => {
                // TODO: render menu, handle menu input
            }
            GamePhase::PlayingWorld | GamePhase::PlayingLocation(_) => {
                handle_input(&mut game)?;
                game.world.update(game.player_pos);
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