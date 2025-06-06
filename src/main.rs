mod core;
mod systems;
mod render;
mod generators;
mod prelude;


use crate::systems::world::{World, Tile, TerrainType};
use prelude::*;

use std::error::Error;
use std::path::Path;
use image::{RgbaImage, Rgba};

use core::{game::Game, input::handle_input};
use render::tui_render::{TuiRenderer,init_terminal, shutdown_terminal, GameTerminal};

pub const MAP_WIDTH: usize = 512;
pub const MAP_HEIGHT: usize = 256;

fn main() {

    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    
}
fn run() -> Result<()> {
    let mut renderer = TuiRenderer::new()?;
    renderer.init();

    let mut player = Player::create_random(MAP_WIDTH / 2, MAP_HEIGHT / 2);
    player.character = Character::create_random(); // or Character::create_human("Name".to_string());

    // Construct initial game state
    let mut game = Game {
        player: player,
        world: World::new(42,MAP_WIDTH, MAP_HEIGHT),
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

    dump_png(&game.world, "world_debug.png").unwrap();

    // Game loop
    loop {
        // First render the current game state
        renderer.render(&game);

        // Then handle the current phase
        match game.phase {
            GamePhase::Menu => {
                // TODO: render menu, handle menu input
            }
            GamePhase::PlayingWorld | GamePhase::PlayingLocation(_) => {
                handle_input(&mut game)?;
                game.world.update(&game.player.world_pos);
            }
            GamePhase::Map => {
                handle_input(&mut game)?;
            }
            GamePhase::GameOver => {
                break;
            }
        }
    }
    // Cleanup and shutdown
    renderer.shutdown();
    Ok(())  // Add explicit Ok return
}


pub fn dump_png(world: &World, path: impl AsRef<Path>) -> Result<()> {
    let w = world.width as u32;
    let h = world.height as u32;
    // Create an empty image buffer (width × height), RGBA8
    let mut img = RgbaImage::new(w, h);

    // For every world coordinate (x, y), pick a color and write it
    for y in 0..h {
        for x in 0..w {
            // Build a Position, get wrapped coordinates just like you do for drawing
            let pos = Position { x: x as usize, y: y as usize };
            let wrapped = world.get_wrapped_coordinates(&pos);
            let tile: &Tile = world.get_tile(&wrapped);

            // Map each TerrainType to an RGBA color
            let px = match tile.terrain {
                TerrainType::Water    => [  0,   0, 200, 255], // dark‐blue
                TerrainType::Plains   => [ 50, 200,  50, 255], // green
                TerrainType::Forest   => [ 10, 150,  10, 255], // darker green
                TerrainType::Desert   => [200, 200, 100, 255], // sandy yellow
                TerrainType::Jungle   => [  0, 120,   0, 255], // deep green
                TerrainType::Snow     => [255, 255, 255, 255], // white
                TerrainType::Swamp    => [ 10, 100,  10, 255], // muddy green
                TerrainType::Mountains=> [100, 100, 100, 255], // gray
                TerrainType::Road     => [150, 150, 150, 255], // light gray
            };

            img.put_pixel(x, y, Rgba(px));
        }
    }

    // Finally, save to disk
    img.save(path)?;
    Ok(())
}


/// Given a 2D array of heights (each in [0.0..1.0]), write it out as a
/// grayscale PNG where 0.0 → black (0) and 1.0 → white (255).
///
/// # Arguments
///
/// * `heights` – a slice‐of‐rows (height first) containing normalized noise values.
/// * `path`    – filesystem path (e.g. "out.png" or PathBuf) where to save.
///
/// # Panics
/// Panics if any value is outside [0.0..1.0]. You can clamp if needed.
pub fn dump_noise_png(heights: &Vec<Vec<f64>>, path: impl AsRef<Path>) -> Result<()> {
    let h = heights.len();
    if h == 0 {
        return Ok(()); 
    }
    let w = heights[0].len();
    // Create an RGBA8 image buffer
    let mut img: RgbaImage = RgbaImage::new(w as u32, h as u32);

    for (y, row) in heights.iter().enumerate() {
        assert!(
            row.len() == w,
            "All rows must have the same width; row {} has length {}, expected {}",
            y,
            row.len(),
            w
        );
        for (x, &val) in row.iter().enumerate() {
            // Expect val in [0.0..1.0]; clamp just in case:
            let clamped = if val < 0.0 {
                0.0
            } else if val > 1.0 {
                1.0
            } else {
                val
            };
            let intensity = (clamped * 255.0).round() as u8;
            // Use grayscale: R=G=B=intensity, A=255
            img.put_pixel(x as u32, y as u32, Rgba([intensity, intensity, intensity, 255]));
        }
    }

    img.save(path)?;
    Ok(())
}
