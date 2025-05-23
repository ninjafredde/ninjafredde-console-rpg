use crate::game::{Game, GamePhase};
use crate::world::World;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};


pub fn handle_world_input(game: &mut Game) -> Result<(), std::io::Error> {
    match event::read()? {
        Event::Key(key_event) => {
            // Only handle key press events, ignore releases
            if key_event.kind != KeyEventKind::Press {
                return Ok(());
            }

            match key_event.code {
                KeyCode::Char('q') => {
                    game.phase = GamePhase::GameOver;
                }
                KeyCode::Char('e') => {
                    handle_interaction(game);
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

fn handle_interaction(game: &mut Game) {
        game.handle_interaction();  // Now we just delegate to Game's handle_interaction

}