use crate::game::{Game, GamePhase};
use crate::world::World;
use crate::location_generator::{LocationMap, LocationTileType};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
pub fn handle_input(game: &mut Game) -> Result<(), std::io::Error> {
    match game.phase {
        GamePhase::PlayingWorld => handle_world_input(game),
        GamePhase::PlayingLocation(_) => handle_location_input(game),
        _ => Ok(()),
    }
}


pub fn handle_world_input(game: &mut Game) -> Result<(), std::io::Error> {
    match event::read()? {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return Ok(());
            }

            match key_event.code {
                KeyCode::Char('q') => {
                    game.phase = GamePhase::GameOver;
                }
                KeyCode::Char('e') => {
                    let current_tile = game.world.get_tile(game.player_pos.0, game.player_pos.1);
                    if current_tile.location.is_some() {
                        game.enter_location();
                    }
                }
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    handle_player_movement(key_event.code, &mut game.player_pos, &game.world);
                    game.update_interaction_prompt();
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_location_input(game: &mut Game) -> Result<(), std::io::Error> {
    match event::read()? {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return Ok(());
            }

            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    game.phase = GamePhase::PlayingWorld;
                }
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    if let GamePhase::PlayingLocation(ref location_map) = game.phase {
                        handle_location_movement(key_event.code, &mut game.player_pos, location_map);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

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

    if world.tiles[new_y as usize][new_x as usize].blocked {
        return;
    }

    *player_pos = (new_x as usize, new_y as usize);
    

}

fn handle_location_movement(
    key: KeyCode,
    player_pos: &mut (usize, usize),
    location_map: &LocationMap,
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

    let new_x = x + dx;
    let new_y = y + dy;

    // Check map boundaries (no wraparound)
    if new_x < 0 || new_y < 0 || 
       new_x >= location_map.width as i32 || 
       new_y >= location_map.height as i32 {
        return;
    }

    // Check if tile is walkable
    let new_tile = &location_map.tiles[new_y as usize][new_x as usize];
    let blocked = matches!(new_tile.tile_type, 
        LocationTileType::Wall | 
        LocationTileType::HumanHouse | 
        LocationTileType::ElfTreehouse | 
        LocationTileType::OrcHut
    );

    if !blocked {
        *player_pos = (new_x as usize, new_y as usize);
    }
}

fn handle_interaction(game: &mut Game) {
        game.handle_interaction();  // Now we just delegate to Game's handle_interaction

}