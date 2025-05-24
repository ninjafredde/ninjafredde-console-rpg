use noise::{NoiseFn, Perlin, Seedable};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::systems::world::{World, Tile, TileGrid, TerrainType};
use crate::systems::location::{Location, Species, Governance, LocationState, Industry};

pub struct WorldGenerator {
    height_noise: Perlin,
    biome_noise: Perlin,
    temperature_noise: Perlin,
    moisture_noise: Perlin,
    size: usize,
    rng: StdRng,
}

impl WorldGenerator {
    pub fn new(seed: u32, size: usize) -> Self {
        WorldGenerator {
            height_noise: Perlin::new(seed),
            biome_noise: Perlin::new(seed + 1),
            temperature_noise: Perlin::new(seed + 2),
            moisture_noise: Perlin::new(seed + 3),
            size,
            rng: StdRng::seed_from_u64(seed as u64),
        }
    }

    pub fn generate(&mut self) -> World {
        let terrain = self.generate_terrain();
        let tiles = self.convert_terrain_to_tiles(&terrain);
        
        World {
            seed: self.rng.gen_range(0..=u64::MAX),
            width: self.size,
            height: self.size,
            wraparound: true,
            tiles,
        }
    }

    pub fn generate_terrain(&self) -> Vec<Vec<TerrainType>> {
        let mut terrain = vec![vec![TerrainType::Plains; self.size]; self.size];

        for y in 0..self.size {
            let y_coord = y as f64;
            for x in 0..self.size {
                let x_coord = x as f64;
                let height = self.get_height(x_coord, y_coord);
                let temperature = self.get_temperature(x_coord, y_coord);
                let moisture = self.get_moisture(x_coord, y_coord);

                terrain[y][x] = self.determine_biome(height, temperature, moisture);
            }
        }

        terrain
    }

    fn get_height(&self, x: f64, y: f64) -> f64 {
        // Adjust scale to get larger features
        let scale = 0.02;
        layered_perlin(
            x * scale, 
            y * scale, 
            &self.height_noise,
            4,    // octaves
            0.5,  // persistence
            2.0   // lacunarity
        )
    }

    fn get_temperature(&self, x: f64, y: f64) -> f64 {
        // Base temperature on latitude (y position)
        let latitude_factor = 1.0 - (2.0 * (y / self.size as f64 - 0.5)).abs();
        
        // Add some noise for variation
        let noise_factor = self.temperature_noise.get([x * 0.03, y * 0.03]);
        
        // Combine latitude and noise (70% latitude, 30% noise)
        0.7 * latitude_factor + 0.3 * ((noise_factor + 1.0) / 2.0)
    }

    fn get_moisture(&self, x: f64, y: f64) -> f64 {
        let scale = 0.03;
        layered_perlin(
            x * scale,
            y * scale,
            &self.moisture_noise,
            3,    // octaves
            0.5,  // persistence
            2.0   // lacunarity
        )
    }

    fn determine_biome(&self, height: f64, temperature: f64, moisture: f64) -> TerrainType {
        // Adjust thresholds for more variety
        if height < 0.45 {
            return TerrainType::Water;  // More water
        }
        if height > 0.7 {
            return TerrainType::Mountains;  // More mountains
        }

        // Then use temperature and moisture to determine the biome
        match (temperature, moisture) {
            // Cold biomes
            (t, _) if t < 0.25 => TerrainType::Snow,
            
            // Hot biomes
            (t, m) if t > 0.5 && m < 0.3 => TerrainType::Desert,
            (t, m) if t > 0.5 && m > 0.6 => TerrainType::Jungle,
            
            // Moderate biomes
            (_, m) if m < 0.4 => TerrainType::Plains,
            (_, m) if m < 0.6 => TerrainType::Forest,
            _ => TerrainType::Swamp,
        }
    }

    fn convert_terrain_to_tiles(&mut self, terrain: &Vec<Vec<TerrainType>>) -> TileGrid {
        terrain.iter().enumerate().map(|(y, row)| {
            row.iter().enumerate().map(|(x, terrain_type)| {
                let blocked = matches!(terrain_type, TerrainType::Water | TerrainType::Mountains);
                let mut location = None;

                if !blocked {
                    let roll = self.rng.gen_range(0.0..1.0);
                    if roll < 0.05 {  // 5% chance for a location
                        location = Some(self.generate_location(*terrain_type));
                    }
                }

                Tile {
                    height: self.get_height(x as f64, y as f64) as f32,
                    terrain: terrain_type.clone(),
                    location,
                    blocked,
                    seen: false,
                }
            }).collect()
        }).collect()
    }

    fn generate_location(&mut self, terrain: TerrainType) -> Location {
        let species = match terrain {
            TerrainType::Plains | TerrainType::Forest => {
                if self.rng.gen_bool(0.5) { Species::Human } else { Species::Elf }
            },
            TerrainType::Mountains => {
                if self.rng.gen_bool(0.5) { Species::Bear } else { Species::Ghost }
            },
            TerrainType::Desert => {
                if self.rng.gen_bool(0.5) { Species::Cat } else { Species::Rat }
            },
            TerrainType::Jungle => Species::Bee,
            TerrainType::Snow => Species::Ghost,
            TerrainType::Swamp => Species::Rat,
            _ => Species::Human,
        };

        // Generate a random state
    let state = match self.rng.gen_range(0..100) {
        0..=10 => LocationState::Ruins,
        11..=20 => LocationState::Abandoned,
        21..=30 => LocationState::Cursed,
        31..=40 => LocationState::Hidden,
        41..=60 => LocationState::Struggling,
        61..=80 => LocationState::Sacred,
        _ => LocationState::Thriving,
    };

    // Size depends on state
    let size = match state {
        LocationState::Ruins | LocationState::Abandoned => self.rng.gen_range(10..30),
        LocationState::Cursed | LocationState::Hidden => self.rng.gen_range(5..15),
        LocationState::Struggling => self.rng.gen_range(30..50),
        LocationState::Sacred => self.rng.gen_range(20..40),
        LocationState::Thriving => self.rng.gen_range(50..100),
    };

// Generate industry based on terrain and add some randomization
    let industry = match terrain {
        TerrainType::Plains => {
            if self.rng.gen_bool(0.7) {
                Industry::Farming
            } else {
                Industry::Trading
            }
        },
        TerrainType::Forest => {
            if self.rng.gen_bool(0.6) {
                Industry::Lumber
            } else {
                Industry::Hunting
            }
        },
        TerrainType::Mountains => {
            if self.rng.gen_bool(0.8) {
                Industry::Mining
            } else {
                Industry::Crafting
            }
        },
        TerrainType::Desert => {
            if self.rng.gen_bool(0.7) {
                Industry::Trading
            } else {
                Industry::Mining // For precious metals/gems
            }
        },
        TerrainType::Jungle => {
            if self.rng.gen_bool(0.6) {
                Industry::Hunting
            } else {
                Industry::Foraging
            }
        },
        TerrainType::Snow => {
            if self.rng.gen_bool(0.7) {
                Industry::Hunting
            } else {
                Industry::Crafting
            }
        },
        TerrainType::Swamp => {
            if self.rng.gen_bool(0.6) {
                Industry::Foraging
            } else {
                Industry::Fishing
            }
        },
        _ => Industry::Trading, // Fallback
    };

    Location {
        name: "Settlement".to_string(), // TODO: Generate names
        species,
        state,
        size,
        governance: Governance::Democracy,
        industry: Industry::Farming,
    }
    }
}

fn layered_perlin(x: f64, y: f64, perlin: &Perlin, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;  // Changed from 5.0 to 1.0 for larger features
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += perlin.get([x * frequency, y * frequency]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    (total / max_value + 1.0) / 2.0
}