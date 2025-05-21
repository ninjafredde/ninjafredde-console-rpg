// src/render.rs

use std::io::{self, Stdout};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Layout, Constraint, Direction},
    widgets::{Paragraph, Block, Borders},
    text::Span,
};

use crate::game::{Game, GamePhase};
use crate::world::{render_tile_map, get_interaction_prompt};

pub fn render_game(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    game: &Game,
) -> Result<(), Box<dyn std::error::Error>> {
    match game.phase {
        GamePhase::PlayingWorld => render_world(terminal, game),
        GamePhase::Menu => render_menu(terminal, game),
        GamePhase::GameOver => render_game_over(terminal, game),
    }
}

fn render_world(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    game: &Game,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(size);

        let bottom_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Min(1),
            ])
            .split(chunks[1]);

        let map_widget = render_tile_map(&game.world, game.player_pos, game.view_radius, "Map");
        f.render_widget(map_widget, chunks[0]);

        let stats = Paragraph::new(Span::from(game.player.stats()));
        f.render_widget(stats, bottom_chunks[0]);

        if let Some(prompt) = get_interaction_prompt(game.world.get_tile(game.player_pos.0, game.player_pos.1)) {
            let action = Paragraph::new(Span::from(prompt));
            f.render_widget(action, bottom_chunks[1]);
        }
    })?;
    Ok(())
}

fn render_menu(
    _terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    _game: &crate::game::Game,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn render_game_over(
    _terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    _game: &crate::game::Game,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}



pub type GameTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn init_terminal() -> io::Result<GameTerminal> {
    use crossterm::{execute, terminal::{EnterAlternateScreen, enable_raw_mode}};

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn shutdown_terminal(terminal: &mut GameTerminal) -> io::Result<()> {
    use crossterm::{execute, terminal::{LeaveAlternateScreen, disable_raw_mode}};
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
