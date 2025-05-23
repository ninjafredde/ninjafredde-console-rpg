use crate::world::{World, TerrainType, Tile};
use crate::location::{Location, LocationState, Species};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Rect, Layout, Constraint, Direction},
    style::{Style, Stylize, Color},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
    Frame,
};
use crate::game::Game;
use std::io;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{DisableMouseCapture, EnableMouseCapture},
};


pub fn render_game(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    game: &Game,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let chunks = create_layout(f.size());
        render_map_widget(f, &game, chunks[0]);      // Map
        render_stats_widget(f, &game, chunks[1]);    // Stats
        render_message_widget(f, &game, chunks[2]);   // Message
        render_action_widget(f, &game, chunks[3]);    // Actions
    })?;
    Ok(())
}

fn create_layout(size: Rect) -> Vec<Rect> {
    // First split the screen vertically into main area and bottom panels
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(70),    // Top section (map + stats)
            Constraint::Length(3),         // Message box
            Constraint::Min(0),            // Action box
        ].as_ref())
        .split(size);

    // Then split the top section horizontally for map and stats
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(80),    // Map area
            Constraint::Percentage(20),    // Stats panel
        ].as_ref())
        .split(vertical_chunks[0]);

    // Combine all chunks into a vector
    vec![
        top_chunks[0],     // Map area [0]
        top_chunks[1],     // Stats panel [1]
        vertical_chunks[1], // Message box [2]
        vertical_chunks[2], // Action box [3]
    ]
}

fn render_map_widget(
    f: &mut Frame<'_>,
    game: &Game,
    area: Rect,
) {
    let map = render_tile_map(&game.world, game.player_pos, game.view_radius, "The World");
    f.render_widget(map, area);
}

fn render_message_widget(
    f: &mut Frame<'_>,
    game: &Game,
    area: Rect,
) {
    if let Some(message) = &game.current_message {
        let message_widget = Paragraph::new(message.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Message"));
        f.render_widget(message_widget, area);
    }
}

fn render_action_widget(
    f: &mut Frame<'_>,
    game: &Game,
    area: Rect,
) {
    let current_tile = game.world.get_tile(game.player_pos.0, game.player_pos.1);
    if let Some(prompt) = get_tile_actions(current_tile) {
        let action_widget = Paragraph::new(prompt)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Actions"));
        f.render_widget(action_widget, area);
    }
}

fn get_tile_actions(tile: &Tile) -> Option<String> {
    let base_actions = "[C] Camp | [I] Inventory | [Q] Quit";
    
    let tile_action = if let Some(location) = &tile.location {
        match location.state {
            LocationState::Thriving | LocationState::Struggling => 
                Some("| [E] Enter Settlement"),
            LocationState::Abandoned | LocationState::Ruins => 
                Some("| [S] Search Area"),
            LocationState::Sacred => 
                Some("| [P] Pray"),
            LocationState::Cursed => 
                Some("| [C] Cleanse"),
            LocationState::Hidden => 
                Some("| [I] Investigate"),
        }
    } else {
        None
    };

    Some(match tile_action {
        Some(action) => format!("{} {}", base_actions, action),
        None => base_actions.to_string(),
    })
}

// Add a new function to render the stats widget
fn render_stats_widget(
    f: &mut Frame<'_>,
    game: &Game,
    area: Rect,
) {
    let stats = vec![
        format!("Name: {}", game.player.name),
        format!("Class: {}", game.player.class),
        format!("HP: {}/{}", game.player.health, game.player.max_health),
        format!("Str: {}", game.player.attack),
        format!("Int: {}", game.player.dodge),
        format!("Dex: {}", game.player.luck),
    ];

    // Convert Vec<String> to Vec<Line>
    let lines: Vec<Line> = stats.iter()
        .map(|s| Line::from(s.as_str()))
        .collect();

    let stats_widget = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Character"));

    f.render_widget(stats_widget, area);
}

const FOG_RADIUS: i32 = 4;
pub fn render_tile_map(
    world: &World,
    player_pos: (usize, usize),
    view_radius: i32,
    title: &str,
) -> Paragraph<'static> {
    let mut lines = Vec::new();
    let (px, py) = (player_pos.0 as i32, player_pos.1 as i32);

    for dy in -view_radius..=view_radius {
        let mut row = Vec::new();
        for dx in -view_radius..=view_radius {
            let (wx, wy) = world.get_wrapped_coordinates(px + dx, py + dy);
            let tile = world.get_tile(wx, wy);
            let (symbol) = tile.appearance();
            let dist = dx*dx + dy*dy;

            let span = if !tile.seen {
                // Unseen tiles - black (hidden)
                Span::styled("  ", Style::default().bg(Color::Black))
            } else if dx == 0 && dy == 0 {
                // Player position - bold white
                Span::styled("@ ", Style::default().bold())
            } else if dist <= FOG_RADIUS*FOG_RADIUS {
                // In view range - colored by terrain/feature
                Span::styled(
                    format!("{} ", symbol),
                    get_terrain_style(tile)
                )
            } else {
                // Out of view range but seen - dimmed
                Span::styled(
                    format!("{} ", symbol),
                    get_terrain_style(tile).dim()
                )
            };
            row.push(span);
        }
        lines.push(Line::from(row));
    }

    let map_title = format!(
        "{} ({}, {}) - {:?}",
        title, player_pos.0, player_pos.1,
        world.get_tile(player_pos.0, player_pos.1).terrain
    );

    Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(map_title))
}

fn get_terrain_style(tile: &Tile) -> Style {
    if let Some(location) = &tile.location {
        return match location.species {
            Species::Human => Style::default().fg(Color::White),
            Species::Orc => Style::default().fg(Color::Red),
            Species::Elf => Style::default().fg(Color::Green),
            Species::Cat => Style::default().fg(Color::Yellow),
            Species::Rat => Style::default().fg(Color::DarkGray),
            Species::Bee => Style::default().fg(Color::Yellow),
            Species::Bear => Style::default().fg(Color::Red),
            Species::Ghost => Style::default().fg(Color::Cyan),
        };
    }

    // Default terrain colors if no location
    let color = match tile.terrain {
        TerrainType::Water => Color::Blue,
        TerrainType::Desert => Color::Yellow,
        TerrainType::Plains => Color::Green,
        TerrainType::Forest => Color::Green,
        TerrainType::Hills => Color::Gray,
        TerrainType::Mountains => Color::White,
    };
    Style::default().fg(color)
}


// Define a type alias for our terminal type
pub type GameTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn init_terminal() -> Result<GameTerminal, Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // Create terminal with crossterm backend
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    Ok(terminal)
}

pub fn shutdown_terminal(terminal: &mut GameTerminal) -> Result<(), Box<dyn std::error::Error>> {
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}