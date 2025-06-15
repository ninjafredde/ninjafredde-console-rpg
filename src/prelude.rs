// Systems
pub use crate::systems::position::Position;
pub use crate::systems::player::Player;
pub use crate::systems::character::Character;
pub use crate::systems::world::{World};

// Core
pub use crate::core::game::{Game, GamePhase};
pub use crate::core::input::handle_input;

// Render
pub use crate::render::{Renderer, tui_render::TuiRenderer};
#[cfg(feature = "bevy-renderer")]
pub use crate::render::bevy_render::BevyRenderer;

// Generators
pub use crate::generators::world_generator::WorldGenerator;
pub use crate::generators::location_generator::LocationGenerator;

// Common types
pub use std::error::Error;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
