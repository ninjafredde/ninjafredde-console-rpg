use crate::core::game::{Game, GamePhase};
use crate::systems::world::World;
use crate::systems::position::Position;
use crate::generators::location_generator::{LocationMap, LocationTileType};
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
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            match key_event.code {
                KeyCode::Char('q') => {
                    game.phase = GamePhase::GameOver;
                }
                KeyCode::Char('e') => {
                    let current_tile = game.world.get_tile(&game.player.world_pos);
                    if current_tile.location.is_some() {
                        game.enter_location();
                    }
                }
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    handle_player_movement(key_event.code, &mut game.player.world_pos, &game.world);
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
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    game.exit_location();
                }
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    if let GamePhase::PlayingLocation(ref location_map) = game.phase {
                        handle_location_movement(key_event.code, &mut game.player.local_pos, location_map);
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
    player_pos: &mut Position,
    world: &World,
) {
    let (dx, dy) = get_direction(key);
    if dx == 0 && dy == 0 { return; }

    let new_x = player_pos.x as i32 + dx;
    let new_y = player_pos.y as i32 + dy;
    
    let (new_x, new_y) = if world.wraparound {
        let w = world.width() as i32;
        let h = world.height() as i32;
        ((new_x + w) % w, (new_y + h) % h)
    } else if !is_in_bounds(new_x, new_y, world.width() as i32, world.height() as i32) {
        return;
    } else {
        (new_x, new_y)
    };

    let new_pos = Position { x: new_x as usize, y: new_y as usize };
    if !world.get_tile(&new_pos).blocked {
        *player_pos = new_pos;
    }
}

fn handle_location_movement(
    key: KeyCode,
    player_pos: &mut Position,
    location_map: &LocationMap,
) {
    let (dx, dy) = get_direction(key);
    if dx == 0 && dy == 0 { return; }

    let new_x = player_pos.x as i32 + dx;
    let new_y = player_pos.y as i32 + dy;

    if !is_in_bounds(new_x, new_y, location_map.width as i32, location_map.height as i32) {
        return;
    }

    let new_pos = Position { x: new_x as usize, y: new_y as usize };
    let new_tile = &location_map.tiles[new_y as usize][new_x as usize];
    
    let blocked = matches!(new_tile.tile_type, 
        LocationTileType::Wall | 
        LocationTileType::HumanHouse | 
        LocationTileType::ElfTreehouse | 
        LocationTileType::OrcHut
    );

    if !blocked {
        *player_pos = new_pos;
    }
}

// Helper functions to reduce duplication
fn get_direction(key: KeyCode) -> (i32, i32) {
    match key {
        KeyCode::Up => (0, -1),
        KeyCode::Down => (0, 1),
        KeyCode::Left => (-1, 0),
        KeyCode::Right => (1, 0),
        _ => (0, 0),
    }
}

fn is_in_bounds(x: i32, y: i32, width: i32, height: i32) -> bool {
    x >= 0 && y >= 0 && x < width && y < height
}