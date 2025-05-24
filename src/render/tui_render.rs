use crate::systems::world::{World, TerrainType, Tile};
use crate::systems::location::{Location, LocationState, Species};
use crate::generators::location_generator::{LocationMap, LocationTileType, FeatureType};

use crate::core::game::{Game, GamePhase};
use crate::systems::player::Player;
use crate::systems::position::Position;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Rect, Layout, Constraint, Direction},
    style::{Style, Stylize, Color},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
    Frame,
};

use std::io;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{DisableMouseCapture, EnableMouseCapture},
};

// Define a type alias for our terminal type
pub type GameTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn render_game(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    game: &Game,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let chunks = create_layout(f.size());
        
        // Render different map based on game phase
        match &game.phase {
            GamePhase::PlayingWorld => {
                render_map_widget(f, game, chunks[0]);
            }
            GamePhase::PlayingLocation(location_map) => {
                let location_view = render_location_map(
                    location_map,
                    &game.player.local_pos,
                    "Location"
                );
                f.render_widget(location_view, chunks[0]);
            }
            GamePhase::Menu => {
                // TODO: Render menu if needed
            }
            GamePhase::GameOver => {
                // TODO: Render game over screen if needed
            }
        }

        // Render common UI elements
        render_stats_widget(f, game, chunks[1]);     // Stats
        render_message_widget(f, game, chunks[2]);    // Message
        render_action_widget(f, game, chunks[3]);     // Actions
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
            Constraint::Length(5),         // Message box
            Constraint::Min(0),            // Action box
        ].as_ref())
        .split(size);

    // Then split the top section horizontally for map and stats
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),    // Map area
            Constraint::Percentage(30),    // Stats panel
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
    let map = render_tile_map(&game.world, &game.player.world_pos, game.view_radius, "The World");
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
                .title("Message"))
                .wrap(Wrap { trim: true });

        f.render_widget(message_widget, area);
    }
}

fn render_action_widget(
    f: &mut Frame<'_>,
    game: &Game,
    area: Rect,
) {
    let current_tile = game.world.get_tile(&game.player.world_pos);
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
        format!("| [E] Enter {} Settlement", location.species)
    } else {
        String::new()
    };

    Some(format!("{} {}", tile_action, base_actions))
}

// Add a new function to render the stats widget
fn render_stats_widget(
    f: &mut Frame<'_>,
    game: &Game,
    area: Rect,
) {
    let stats = vec![
        format!("Name: {}", game.player.character.name),
        format!("Species: {}", game.player.character.species),
        format!("Class: {}", game.player.character.class),
        format!("HP: {}/{}", game.player.character.health, game.player.character.max_health),
        format!("Str: {}", game.player.character.attack),
        format!("Int: {}", game.player.character.dodge),
        format!("Dex: {}", game.player.character.luck),
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
const RENDER_RADIUS: i32 = 10;  // Add this constant

// render world map
pub fn render_tile_map(
    world: &World,
    player_pos: &Position,
    view_radius: i32,
    title: &str,
) -> Paragraph<'static> {
    let mut lines = Vec::new();
    let (px, py) = (player_pos.x as i32, player_pos.y as i32);

    for dy in -RENDER_RADIUS..=RENDER_RADIUS {
        let mut row = Vec::new();
        for dx in -RENDER_RADIUS..=RENDER_RADIUS {
            let world_x = px + dx;
            let world_y = py + dy;
            
            // Create position for current tile we're trying to render
            let current_pos = Position {
                x: world_x as usize,
                y: world_y as usize,
            };
            
            // Get wrapped coordinates for this position
            let wrapped_pos = world.get_wrapped_coordinates(&current_pos);
            let tile = world.get_tile(&wrapped_pos);
            let symbol = tile.appearance();
            let dist = dx*dx + dy*dy;
            
            // fog of war
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
        title, player_pos.x, player_pos.y,
        world.get_tile(player_pos).terrain
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
        TerrainType::Plains => Color::LightGreen,
        TerrainType::Forest => Color::Green,
        //TerrainType::Hills => Color::Gray,
        TerrainType::Mountains => Color::Gray,
        TerrainType::Snow => Color::White,
        TerrainType::Jungle => Color::Green,
        TerrainType::Swamp => Color::Green,
    };
    Style::default().fg(color)
}


fn render_location_map(
    map: &LocationMap,
    player_pos: &Position,
    title: &str,
) -> Paragraph<'static> {
    let mut lines = Vec::new();
    let (px, py) = (player_pos.x as i32, player_pos.y as i32);

    for dy in -RENDER_RADIUS..=RENDER_RADIUS {
        let mut row = Vec::new();
        for dx in -RENDER_RADIUS..=RENDER_RADIUS {
            let dist = dx*dx + dy*dy;
            let world_x = px + dx;
            let world_y = py + dy;
            
            let span = if dist > RENDER_RADIUS * RENDER_RADIUS {
                // Out of view range
                Span::styled("  ", Style::default().fg(Color::DarkGray))
            } else if world_x < 0 || world_y < 0 || 
                      world_x >= map.width as i32 || 
                      world_y >= map.height as i32 {
                // Out of bounds - show empty space
                Span::styled("  ", Style::default().fg(Color::DarkGray))
            } else {
                let current_pos = Position {
                    x: world_x as usize,
                    y: world_y as usize,
                };

                if current_pos.x == player_pos.x && current_pos.y == player_pos.y {
                    // Player position
                    Span::styled("@ ", Style::default().bold())
                } else {
                    let tile = &map.tiles[world_y as usize][world_x as usize];
                    if let Some(feature) = &tile.feature {
                        // Features
                        let symbol = match feature.feature_type {
                            FeatureType::Market => "M ",
                            FeatureType::Temple => "T ",
                            FeatureType::Tavern => "A ",
                            FeatureType::Blacksmith => "B ",
                            FeatureType::Garden => "* ",
                            FeatureType::TrainingGround => "X ",
                            FeatureType::Storage => "S ",
                        };
                        Span::styled(symbol, Style::default().fg(Color::Yellow))
                    } else {
                        // Regular tiles
                        let (symbol, style) = match tile.tile_type {
                            LocationTileType::Ground => (". ", Style::default().fg(Color::DarkGray)),
                            LocationTileType::Wall => ("# ", Style::default().fg(Color::White)),
                            LocationTileType::Water => ("~ ", Style::default().fg(Color::Blue)),
                            LocationTileType::HumanRoad => ("= ", Style::default().fg(Color::Gray)),
                            LocationTileType::ElfPath => ("- ", Style::default().fg(Color::Green)),
                            LocationTileType::OrcTrail => (": ", Style::default().fg(Color::Red)),
                            LocationTileType::HumanHouse => ("H ", Style::default().fg(Color::White)),
                            LocationTileType::ElfTreehouse => ("T ", Style::default().fg(Color::Green)),
                            LocationTileType::OrcHut => ("O ", Style::default().fg(Color::Red)),
                            LocationTileType::Trading => ("$ ", Style::default().fg(Color::Yellow)),
                            LocationTileType::Shrine => ("^ ", Style::default().fg(Color::Magenta)),
                        };
                        Span::styled(symbol, style)
                    }
                }
            };
            row.push(span);
        }
        lines.push(Line::from(row));
    }

    let location_title = format!("{} ({}, {}) - Points of Interest:", 
        title, player_pos.x, player_pos.y);
    let mut text = Text::from(lines);
    
    // Add nearby points of interest only
    let mut poi_lines = Vec::new();
    for poi in &map.points_of_interest {
        // Update to use Position fields
        let dx = poi.position.x as i32 - px;
        let dy = poi.position.y as i32 - py;
        if dx*dx + dy*dy <= RENDER_RADIUS*RENDER_RADIUS {
            poi_lines.push(Line::from(format!(
                "{} at ({}, {})",
                poi.feature.name,
                poi.position.x,
                poi.position.y
            )));
        }
    }
    text.extend(poi_lines);

    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(location_title))
}


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