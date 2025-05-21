
use ratatui::{
    layout::{Layout, Constraint, Direction},
    widgets::{Paragraph, Block, Borders},
    style::{Style, Color, Stylize},
    text::{Text, Line, Span},
};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use noise::{NoiseFn, Perlin};

#[derive(Clone, Copy, PartialEq, Debug)]

pub enum TerrainType {
    Plains,
    Forest,
    Desert,
    Hills,
    Mountains,
    Water,
}
#[derive(Clone, Copy, PartialEq)]
pub enum FeatureType {
    None,
    Town,
    Dungeon,
    Mine,
    Shrine,
    Ruins,
}
#[derive(Clone, Copy, PartialEq)]
pub struct Tile {
    pub height: f32,
    pub terrain: TerrainType,
    pub feature: FeatureType,
    pub blocked: bool,
    pub seen: bool,

    
}

impl Tile {
    pub fn appearance(&self) -> (char, Color) {
        if self.feature != FeatureType::None {
            return match self.feature {
                FeatureType::Town => ('T', Color::Yellow),
                FeatureType::Mine => ('M', Color::Gray),
                FeatureType::Dungeon => ('D', Color::Magenta),
                FeatureType::Shrine => ('S', Color::Cyan),
                FeatureType::Ruins => ('R', Color::DarkGray),
                _ => ('?', Color::White),
            };
        }

        match self.terrain {
            TerrainType::Water => ('~', Color::Blue),
            TerrainType::Desert => ('.', Color::Yellow),
            TerrainType::Plains => (',', Color::Green),
            TerrainType::Forest => ('♣', Color::Green),
            TerrainType::Hills => ('^', Color::DarkGray),
            TerrainType::Mountains => ('▲', Color::Gray),
        }
    }
}

pub type TileGrid = Vec<Vec<Tile>>;

pub struct World {
    pub seed: u64,
    pub width: usize,
    pub height: usize,
    pub wraparound: bool,
    pub tiles: TileGrid,
}

impl World {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_symbol(&self, x: usize, y: usize) -> char {
        match self.tiles[y][x].terrain {
            TerrainType::Water => '~',
            TerrainType::Desert => '.',
            TerrainType::Plains => ',',
            TerrainType::Forest => '♣',
            TerrainType::Hills => '^',
            TerrainType::Mountains => '▲',
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> &Tile {
    &self.tiles[y][x]
}

}
pub const MAP_WIDTH: usize = 128;
pub const MAP_HEIGHT: usize = 128;

pub fn get_interaction_prompt(tile: &Tile) -> Option<&'static str> {
    match tile.feature {
        FeatureType::Shrine => Some("(P) Pray"),
        FeatureType::Town => Some("(E) Enter Town"),
        FeatureType::Dungeon => Some("(D) Descend"),
        FeatureType::Ruins => Some("(S) Search"),
        FeatureType::Mine => Some("(M) Mine"),
        FeatureType::None => None,
    }
}


pub fn generate_world_map(seed: u32) -> World {
    let perlin = Perlin::new(seed); // Deterministic noise
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let width = MAP_WIDTH;
    let height = MAP_HEIGHT;

      let mut tiles: TileGrid = vec![vec![Tile {
        height: 0.0,
        terrain: TerrainType::Plains,
        feature: FeatureType::None,
        blocked: false,
        seen: false,
    }; width]; height];



    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 / width as f64;
            let ny = y as f64 / height as f64;

            // Scale for variation
            //let frequency = 2.0;
            let height_val = layered_perlin(nx, ny, &perlin, 4, 0.4, 2.0) as f32;

            let terrain = if height_val < 0.45 {
                TerrainType::Water
            } else if height_val < 0.55 {
                TerrainType::Desert
            } else if height_val < 0.6 {
                TerrainType::Plains
            } else if height_val < 0.7 {
                TerrainType::Forest
            } else if height_val < 0.8 {
                TerrainType::Hills
            } else {
                TerrainType::Mountains
            };

            let blocked = matches!(terrain, TerrainType::Water | TerrainType::Mountains);

            // randomly add features
            let mut feature = FeatureType::None;


            if !blocked {
                // 1% chance for a town, 2% for a mine in desert or hills, 3% for a dungeon
                // 0.5% for shrine, 0.5% for ruins
            let roll = rng.gen_range(0.0..1.0);
                
                feature = if roll < 0.01 {
                    FeatureType::Town // 1%
                } else if (terrain == TerrainType::Hills || terrain == TerrainType::Desert) && roll < 0.02 {
                    FeatureType::Mine // 1% in desert, 2% in hills
                } else if roll < 0.03 {
                    FeatureType::Dungeon // 3%
                } else if roll < 0.035 {
                    FeatureType::Shrine
                } else if roll < 0.04 {
                    FeatureType::Ruins
                } else {
                    FeatureType::None
                };
            }


            tiles[y][x] = Tile {
                height: height_val,
                terrain,
                feature,
                blocked,
                seen: false,
                
            };
        }
    }

    World {
        seed: seed as u64,
        width,
        height,
        wraparound: true,
        tiles,
    }
}

// Deterministic local seed function
pub fn local_seed(global_seed: u64, world_x: usize, world_y: usize) -> u64 {
    global_seed ^ ((world_x as u64) << 32 | (world_y as u64))
}


//rendering
pub fn render_tile_map(
    map: &World,
    player_pos: (usize, usize),
    view_radius: i32,
    title: &str,
) -> Paragraph<'static> {
    
    let mut lines = Vec::new();
    let (px, py) = (player_pos.0 as i32, player_pos.1 as i32);
    

    for dy in -view_radius..=view_radius {
    let mut row = Vec::new();
    for dx in -view_radius..=view_radius {
        let x = px + dx;
        let y = py + dy;

        let (wrapped_x, wrapped_y, in_bounds) = if map.wraparound {
            let w = map.width() as i32;
            let h = map.height() as i32;
            let wx = (x.rem_euclid(w)) as usize;
            let wy = (y.rem_euclid(h)) as usize;
            (wx, wy, true)
        } else if x < 0 || y < 0 || x >= map.width() as i32 || y >= map.height() as i32 {
            (0, 0, false)
        } else {
            (x as usize, y as usize, true)
        };

        let tile = if in_bounds {
            Some(&map.tiles[wrapped_y][wrapped_x])
        } else {
            None
        };

        let span = match tile {
            Some(t) if dx == 0 && dy == 0 => {
                Span::styled("@ ", Style::default().fg(Color::White).bold())
            }
            Some(t) => {
                let (symbol, color) = t.appearance();
                if dx.abs() <= view_radius && dy.abs() <= view_radius {
                    // CURRENTLY visible
                    Span::styled(format!("{} ", symbol), Style::default().fg(color))
                } else if t.seen {
                    // Seen before, but not visible now
                    Span::styled(format!("{} ", symbol), Style::default().fg(color).dim())
                } else {
                    // Never seen
                    Span::raw("  ")
                }
            }
            None => Span::raw("  "),
        };


        row.push(span);
    }
    lines.push(Line::from(row));
}


    let text = Text::from(lines);
    
    let tile = &map.tiles[player_pos.1][player_pos.0];
    let map_title = format!("World ({}, {}) - {:?}", player_pos.0, player_pos.1, tile.terrain);

    //let map_title = format!("World ({}, {}) - {}", player_pos.0, player_pos.1, tile.terrain.tostring());

    Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(Span::from(map_title)))
}
fn layered_perlin(x: f64, y: f64, perlin: &Perlin, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
    let mut total = 0.0;
    let mut frequency = 7.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += perlin.get([x * frequency, y * frequency]) * amplitude;

        max_value += amplitude;
        amplitude *= persistence; // drop-off factor
        frequency *= lacunarity; // frequency increase
    }

    // Normalize to -1..1, then remap to 0..1
    (total / max_value + 1.0) / 2.0
}