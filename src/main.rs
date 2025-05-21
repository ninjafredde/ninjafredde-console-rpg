mod character;
use character::{Character,Player};
mod world;
use world::World;
use world::{render_tile_map, get_interaction_prompt};
use world::{MAP_WIDTH, MAP_HEIGHT};
mod game;
use game::{Game, GamePhase};

mod render;
use render::{render_game, shutdown_terminal, init_terminal, GameTerminal};

use rand::{Rng};


use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};


use std::{io, time::Duration, time::Instant};

fn main() {

    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal: GameTerminal = init_terminal()?;    // Start menu
    // Game title
    // (C)reate player
    // (S)tart game
    // (Q)uit

    // Construct initial game state
    let mut game = Game {
        player: create_player(),
        player_pos: (MAP_WIDTH / 2, MAP_HEIGHT / 2),
        world: world::generate_world_map(12345),
        view_radius: 10,
        phase: GamePhase::PlayingWorld,
        last_input_time: Instant::now(),
        input_cooldown: Duration::from_millis(50),
    };

    // Game loop
    loop {
        match game.phase {
            GamePhase::Menu => {
                // TODO: render menu, handle menu input
            }
            GamePhase::PlayingWorld => {
                update_visibility(&mut game.world, game.player_pos, game.view_radius);
                handle_world_input(&mut game);
                println!("Player at: {:?}", game.player_pos);

                render_game(&mut terminal, &game);
            }
            GamePhase::GameOver => {
                shutdown_terminal(&mut terminal);
                std::process::exit(0);
            }
        }
    }

    shutdown_terminal(&mut terminal);
}
pub fn create_player() -> Character {
    let character_name = "Player1".to_string();
    let luck_amount = rand::thread_rng().gen_range(2..6);
    let characters = [
        Character::new("".to_string(), "Cleric".to_string(), 7, 5, 6, luck_amount),
        Character::new("".to_string(), "Warrior".to_string(), 10, 5, 5, luck_amount),
        Character::new("".to_string(), "Hunter".to_string(), 5, 7, 7, luck_amount),
        Character::new("".to_string(), "Wizard".to_string(), 3, 10, 5, luck_amount),
        Character::new("".to_string(), "Thief".to_string(), 4, 5, 6, luck_amount),
    ];

    let index = rand::thread_rng().gen_range(0..characters.len());

    let mut player = characters[index].select(character_name.clone());
    let player_pos = (MAP_WIDTH / 2, MAP_HEIGHT / 2);

    player.name = character_name;
    player
}

    fn handle_world_input(game: &mut Game) {

        if event::poll(Duration::from_millis(500)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                let now = Instant::now();
                if now.duration_since(game.last_input_time) < game.input_cooldown {
                    return;
                }

                game.last_input_time = now;

                match key_event.code {
                    KeyCode::Char('q') => {
                        game.phase = GamePhase::GameOver;
                    }
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                        println!("Calling movement");

                        handle_player_movement(key_event.code, &mut game.player_pos, &game.world);
                    }
                    _ => {}
                }
            }
        }
    }



// should probably be all input handling, based on gamestate
fn handle_player_movement(
    key: KeyCode,
    player_pos: &mut (usize, usize),
    world: &World,
) {

    let (x, y) = *player_pos;
    let (x, y) = (x as i32, y as i32);
    let (dx, dy) = match key {
        KeyCode::Up => (0, -1),
        KeyCode::Down => (0, 1),
        KeyCode::Left => (-1, 0),
        KeyCode::Right => (1, 0),
        _ => return,
    };

    let (mut new_x, mut new_y) = (x + dx, y + dy);

    if world.wraparound {
        let w = world.width() as i32;
        let h = world.height() as i32;
        new_x = (new_x + w) % w;
        new_y = (new_y + h) % h;
    } else {
        if new_x < 0 || new_y < 0 || new_x >= world.width() as i32 || new_y >= world.height() as i32 {
            return;
        }
    }

    //if world.tiles[new_y as usize][new_x as usize].blocked {
      //  return;
    //}

    *player_pos = (new_x as usize, new_y as usize);

}


// needs revision. not working
fn update_visibility(world: &mut World, player_pos: (usize, usize), radius: i32) {
    let (px, py) = (player_pos.0 as i32, player_pos.1 as i32);

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = px + dx;
            let y = py + dy;

            if world.wraparound {
                let wx = x.rem_euclid(world.width() as i32) as usize;
                let wy = y.rem_euclid(world.height() as i32) as usize;
                world.tiles[wy][wx].seen = true;
            } else if x >= 0 && y >= 0 && x < world.width() as i32 && y < world.height() as i32 {
                world.tiles[y as usize][x as usize].seen = true;
            }
        }
    }
}