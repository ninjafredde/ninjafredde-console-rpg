use noise::{NoiseFn, RidgedMulti,Perlin, MultiFractal, Seedable};

use rand::RngCore;

use rand::{Rng, SeedableRng, rngs::StdRng};

use rand::seq::SliceRandom; // ← add this at the top
use std::collections::{VecDeque, HashSet};
use crate::dump_noise_png;
use crate::systems::world::{World, Tile, TileGrid, TerrainType};
use crate::systems::location::{Location, Species, Governance, LocationState, Industry};

pub struct WorldGenerator {
    height_noise: Perlin,
    biome_noise: Perlin,
    temperature_noise: Perlin,
    moisture_noise: Perlin,
    width: usize,
    height: usize,
    rng: StdRng,
    pub river_mouths: Vec<(usize, usize)>,
    ridged_noise: RidgedMulti<Perlin>,

}

const OCEAN_LEVEL: f64 = 0.5; // Adjusted ocean level for more water

/// Given a list of all candidate sources `(x,y)`, return a subset
    /// such that no two chosen sources are within `min_dist` tiles (Chebyshev).
    ///
    /// - `all_sources`: e.g. all (x,y) where height > 0.8
    /// - `min_dist`: minimal Chebyshev distance between any two accepted sources
    /// Free‐function helper: filter out any two sources closer than `min_dist` (Chebyshev).
fn filter_sources_by_distance(
    rng: &mut impl Rng,
    all_sources: &[(usize, usize)],
    min_dist: usize,
) -> Vec<(usize, usize)> {
    let mut chosen = Vec::new();
    let mut pool = all_sources.to_vec();
    pool.shuffle(rng);

    'outer: for &(sx, sy) in &pool {
        for &(cx, cy) in &chosen {
            let dx = (cx as isize - sx as isize).abs() as usize;
            let dy = (cy as isize - sy as isize).abs() as usize;
            if std::cmp::max(dx, dy) < min_dist {
                continue 'outer;
            }
        }
        chosen.push((sx, sy));
    }
    chosen
}


impl WorldGenerator {
     pub fn new(seed: u32, width: usize, height: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed as u64);
        WorldGenerator {
            height_noise: Perlin::new(seed),
            biome_noise: Perlin::new(seed + 1),
            temperature_noise: Perlin::new(seed + 2),
            moisture_noise: Perlin::new(seed + 3),
            width,
            height,
            rng: StdRng::seed_from_u64(seed as u64),
            river_mouths: Vec::new(),
            ridged_noise: RidgedMulti::new(seed + 4)
                .set_octaves(6)
                .set_frequency(1.0)
                .set_lacunarity(2.0)
                .set_persistence(0.5)
                .set_attenuation(2.0),

        }
    }
    pub fn generate(&mut self) -> World {
        // 1) FULL HEIGHTMAP (height rows × width columns)
        let heights: Vec<Vec<f64>> = (0..self.height).map(|y| {
            let y_f = y as f64;
            (0..self.width).map(|x| {
                let x_f = x as f64;
                self.get_noise(x_f, y_f, 0.01, 0.7,2.0,6)
            }).collect()
        }).collect();
        
        // 2) BASE TERRAIN
        let mut terrain_map = vec![vec![TerrainType::Plains; self.width]; self.height];
        for y in 0..self.height {
            let y_f = y as f64;
            for x in 0..self.width {
                let x_f = x as f64;
                let h = heights[y][x];
                let t = self.get_temperature(x_f, y_f);
                let m = self.get_noise(x_f, y_f,0.03, 0.75,2.0,3); // Adjusted moisture scale
                terrain_map[y][x] = self.determine_biome(h, t, m);
            }
        }
        dump_noise_png(&heights, "heightmap.png").expect("Failed to save heightmap");
        
        let freq = 0.02; // adjust to taste: larger → thinner ridges
        let threshold = 0.6; // pick a high threshold to isolate the ridges

        let mut raw_river: Vec<Vec<f64>> = (0..self.height)
            .map(|y: usize| {
                (0..self.width)
                    .map(|x: usize| {
                        // Sample our RidgedMulti<Perlin> at (x*freq, y*freq)
                        let raw = self.ridged_noise.get([x as f64 * freq, y as f64 * freq]);
                        // raw ∈ [-1.0..1.0], remap to [0.0..1.0]
                        let ridged = (raw + 1.0) * 0.5;

                        if ridged > threshold {
                            self.river_mouths.push((x, y));
                            1.0 // mark as river
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .collect();
            dump_noise_png(&raw_river, "river_map.png").expect("Failed to save river map");

            // 2) Convert that into a boolean `river_map`: true wherever ridged > threshold
            let river_map: Vec<Vec<bool>> = raw_river
                .iter()
                .map(|row| row.iter().map(|&v| v > 0.5).collect())
                .collect();


        // 4) BUILD TILE GRID (override terrain→Water wherever river_map==true)
        let tiles = self.convert_terrain_to_tiles(&terrain_map, &heights, &river_map);

        // 5) PACKAGE WORLD
        let combined_seed = {
            let high = self.rng.next_u32() as u64;
            let low  = self.rng.next_u32() as u64;
            (high << 32) ^ low
        };
        World {
            seed: combined_seed,
            width: self.width,
            height: self.height,
            wraparound: true,
            tiles,
        }
    
    }

    // turned into a generic noise function
    fn get_noise(&self, x: f64, y: f64, scale:f64, persistence:f64, lacunarity:f64, octaves: usize) -> f64 {
        // Adjust scale to get larger features
        //let scale = 0.02;
        layered_perlin(
            x * scale, 
            y * scale, 
            &self.height_noise,
            octaves,    // octaves
            persistence,  // persistence
            lacunarity   // lacunarity
        )
    }

    // get temperature based on latitude and noise
    fn get_temperature(&self, x: f64, y: f64) -> f64 {
        let latitude_factor = 1.0 - (2.0 * (y / self.height as f64 - 0.5)).abs();
        let noise_factor = self.temperature_noise.get([x * 0.1, y * 0.1]);
        0.8 * latitude_factor + 0.2 * ((noise_factor + 1.0) / 2.0)
    }

    /// Determine the biome based on height, temperature, and moisture.
    fn determine_biome(&self, height: f64, temperature: f64, moisture: f64) -> TerrainType {
        // Adjust thresholds for more variety
        if height < OCEAN_LEVEL {
            return TerrainType::Water;  // More water
        }
        if height > 0.78 {
            return TerrainType::Mountains;  // More mountains
        }

        // Then use temperature and moisture to determine the biome
        match (temperature, moisture) {
            // Cold biomes
            (t, _) if t < 0.25 => TerrainType::Snow,
            
            // Hot biomes
            (t, m) if t > 0.5 && m < 0.4 => TerrainType::Desert,
            (t, m) if t > 0.5 && m > 0.6 => TerrainType::Jungle,
            
            // Moderate biomes
            (_, m) if m < 0.4 => TerrainType::Plains,
            (_, m) if m < 0.6 => TerrainType::Forest,
            _ => TerrainType::Swamp,
        }
    }
    
    


     fn convert_terrain_to_tiles(
        &mut self,
        terrain: &Vec<Vec<TerrainType>>,
        heights: &Vec<Vec<f64>>,
        river_map: &Vec<Vec<bool>>,    // <— new parameter
    ) -> TileGrid {
        let w = self.width;
        let h = self.height;

        (0..h)
            .map(|y| {
                (0..w)
                    .map(|x| {
                        // If river_map[y][x] is true, force water (or a River terrain):
                        let final_terrain = if river_map[y][x] {
                            TerrainType::Water
                        } else {
                            terrain[y][x]
                        };

                        let blocked =
                            matches!(final_terrain, TerrainType::Water | TerrainType::Mountains);
                        let mut location = None;
                        if !blocked {
                            let roll = self.rng.gen_range(0.0..1.0);
                            if roll < 0.05 {
                                location = Some(self.generate_location(final_terrain));
                            }
                        }

                        Tile {
                            height: heights[y][x] as f32,
                            terrain: final_terrain,
                            location,
                            blocked,
                            seen: false,
                        }
                    })
                    .collect()
            })
            .collect()
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

fn generate_river_map(&mut self) -> Vec<Vec<bool>> {
        let w = self.width;
        let h = self.height;
        let mut river_map = vec![vec![false; w]; h];
        self.river_mouths.clear();

        for y in 0..h {
            for x in 0..w {
                // Sample `ridged_noise` (it returns [-1.0..1.0])
                let val = self.ridged_noise.get([x as f64 * 0.02, y as f64 * 0.02]);
                let ridged = (val + 1.0) * 0.5; // map to [0..1]
                if ridged > 0.7 {
                    river_map[y][x] = true;
                    self.river_mouths.push((x, y));
                }
            }
        }
        river_map
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


