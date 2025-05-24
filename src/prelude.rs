// Systems
pub use crate::systems::position::Position;
pub use crate::systems::player::Player;
pub use crate::systems::character::Character;
pub use crate::systems::world::{World, MAP_WIDTH, MAP_HEIGHT};

// Core
pub use crate::core::game::{Game, GamePhase};
pub use crate::core::input::handle_input;

// Render
pub use crate::render::tui_render::{render_game, init_terminal, shutdown_terminal};

// Generators
pub use crate::generators::world_generator::WorldGenerator;
pub use crate::generators::location_generator::LocationGenerator;

// Common types
pub use std::error::Error;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;