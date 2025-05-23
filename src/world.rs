use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use noise::{NoiseFn, Perlin};
use crate::location::{Location, Species, Governance, LocationState, Industry};

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
#[derive(Clone)]
pub struct Tile {
    pub height: f32,
    pub terrain: TerrainType,
    pub location: Option<Location>,  // Replace feature with location
    pub blocked: bool,
    pub seen: bool,
}

impl Tile {
    pub fn appearance(&self) -> char {
        if let Some(location) = &self.location {
            return match location.species {
                Species::Human => 'H',
                Species::Orc => 'O',
                Species::Elf => 'E',
                Species::Cat => 'C',
                Species::Rat => 'R',
                Species::Bee => 'B',
                Species::Bear => 'Ʊ',
                Species::Ghost => 'G',
            };
        }

        match self.terrain {
            TerrainType::Water => '~',
            TerrainType::Desert => '.',
            TerrainType::Plains => ',',
            TerrainType::Forest => '♣',
            TerrainType::Hills => '^',
            TerrainType::Mountains => '▲',
        }
    }
}


pub type TileGrid = Vec<Vec<Tile>>;
const FOG_RADIUS: i32 = 4;
pub struct World {
    pub seed: u64,
    pub width: usize,
    pub height: usize,
    pub wraparound: bool,
    pub tiles: TileGrid,
}

impl World {
    // Read-only methods (take &self)
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_tile(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[y][x]
    }

    pub fn get_wrapped_coordinates(&self, x: i32, y: i32) -> (usize, usize) {
        if self.wraparound {
            let wx = x.rem_euclid(self.width() as i32) as usize;
            let wy = y.rem_euclid(self.height() as i32) as usize;
            (wx, wy)
        } else {
            (x.clamp(0, self.width() as i32 - 1) as usize,
             y.clamp(0, self.height() as i32 - 1) as usize)
        }
    }

    // State-changing methods (take &mut self)
    pub fn update(&mut self, player_pos: (usize, usize)) {
        self.update_visibility(player_pos, FOG_RADIUS);
        // Other world updates here...
    }

    fn update_visibility(&mut self, (px, py): (usize, usize), radius: i32) {
    let (px, py) = (px as i32, py as i32);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            // Only mark as seen if within circular radius, not square
            let dist = dx*dx + dy*dy;
            if dist <= radius*radius {
                let (wx, wy) = self.get_wrapped_coordinates(px + dx, py + dy);
                self.tiles[wy][wx].seen = true;
            }
        }
    }
}
pub fn find_nearest_species(
        &self,
        start: (usize, usize),
        target_species: Species,
    ) -> Option<(usize, usize)> {
        let (sx, sy) = (start.0 as i32, start.1 as i32);
        let mut closest: Option<((usize, usize), i32)> = None;

        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if let Some(location) = &tile.location {
                    if location.species == target_species {
                        let dx = sx - x as i32;
                        let dy = sy - y as i32;
                        let dist = dx.abs() + dy.abs(); // Manhattan distance

                        match closest {
                            Some((_, best_dist)) if dist < best_dist => {
                                closest = Some(((x, y), dist));
                            }
                            None => {
                                closest = Some(((x, y), dist));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        closest.map(|(pos, _)| pos)
    }
    
    pub fn get_interaction_prompt(&self, tile: &Tile) -> Option<String> {
        tile.location.as_ref().map(|loc| loc.generate_description())
    }
}



pub const MAP_WIDTH: usize = 128;
pub const MAP_HEIGHT: usize = 128;




// Generates a world map using Perlin noise
// This should be its own module
pub fn generate_world_map(seed: u32) -> World {
    let perlin = Perlin::new(seed); // Deterministic noise
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let width = MAP_WIDTH;
    let height = MAP_HEIGHT;

      let mut tiles: TileGrid = vec![vec![Tile {
        height: 0.0,
        terrain: TerrainType::Plains,
        location: None,
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
            let mut location = None;

            if !blocked {
                let roll = rng.gen_range(0.0..1.0);
                if roll < 0.05 {  // 5% chance for a location
                    location = Some(generate_location(&mut rng, terrain));
                }
            }

            tiles[y][x] = Tile {
                height: height_val,
                terrain,
                location,
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


fn generate_location(rng: &mut StdRng, terrain: TerrainType) -> Location {
    let species = match terrain {
        TerrainType::Mountains => vec![Species::Orc, Species::Bear],
        TerrainType::Forest => vec![Species::Elf, Species::Bear, Species::Bee],
        TerrainType::Plains => vec![Species::Human, Species::Cat],
        TerrainType::Hills => vec![Species::Orc, Species::Human],
        TerrainType::Desert => vec![Species::Rat, Species::Ghost],
        TerrainType::Water => vec![Species::Ghost],
    };

    // Select industry based on terrain and some randomness
    let primary_industries = match terrain {
        TerrainType::Plains => vec![
            (Industry::Farming, 0.6),
            (Industry::Trading, 0.3),
            (Industry::Crafting, 0.1),
        ],
        TerrainType::Forest => vec![
            (Industry::Lumber, 0.5),
            (Industry::Hunting, 0.3),
            (Industry::Crafting, 0.2),
        ],
        TerrainType::Hills => vec![
            (Industry::Mining, 0.6),
            (Industry::Crafting, 0.3),
            (Industry::Trading, 0.1),
        ],
        TerrainType::Desert => vec![
            (Industry::Trading, 0.4),
            (Industry::Mining, 0.3),
            (Industry::Research, 0.3),
        ],
        _ => vec![(Industry::Trading, 1.0)],
    };

   let industry = {
    let roll = rng.gen_range(0.0..1.0);
    let mut cumulative = 0.0;
    let mut selected = Industry::Trading; // Default value

    for (ind, chance) in primary_industries {
        cumulative += chance;
        if roll < cumulative {
            selected = ind;
            break; // Exit loop once we find our industry
        }
    }
    selected
};

    Location {
        name: "Unnamed".to_string(),
        species: species[rng.gen_range(0..species.len())].clone(),
        governance: match rng.gen_range(0..6) {
            0 => Governance::Monarchy,
            1 => Governance::Democracy,
            2 => Governance::Theocracy,
            3 => Governance::Anarchy,
            4 => Governance::Hivemind,
            _ => Governance::Council,
        },
        state: match rng.gen_range(0..7) {
            0 => LocationState::Thriving,
            1 => LocationState::Struggling,
            2 => LocationState::Abandoned,
            3 => LocationState::Ruins,
            4 => LocationState::Cursed,
            5 => LocationState::Sacred,
            _ => LocationState::Hidden,
        },
        size: rng.gen_range(10..1000),
        industry, // Add the industry field
    }
}



fn layered_perlin(x: f64, y: f64, perlin: &Perlin, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
    let mut total = 0.0;
    let mut frequency = 5.0;
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