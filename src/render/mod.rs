use crate::core::game::Game;

pub trait Renderer {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn render(&mut self, game: &Game) -> Result<(), Box<dyn std::error::Error>>;
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

pub mod tui_render;

#[cfg(feature = "bevy-renderer")]
pub mod bevy_render;
