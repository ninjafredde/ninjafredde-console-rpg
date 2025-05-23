use crate::world_generator::WorldGenerator;
use crate::location::{Location, Species};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerrainType {
    Water,
    Plains,
    Forest,
    Mountains,
    Desert,
    Snow,
    Jungle,
    Swamp,
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
    pub location: Option<Location>,
    pub blocked: bool,
    pub seen: bool,
}

pub struct World {
    pub seed: u64,
    pub width: usize,
    pub height: usize,
    pub wraparound: bool,
    pub tiles: TileGrid,
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
            TerrainType::Water => 'w',
            TerrainType::Plains => ',',
            TerrainType::Forest => 'p',
            TerrainType::Mountains => '^',
            TerrainType::Desert => '.',
            TerrainType::Snow => '*',
            TerrainType::Jungle => 'd',
            TerrainType::Swamp => 's',
        }
    }
}

pub type TileGrid = Vec<Vec<Tile>>;
const FOG_RADIUS: i32 = 4;

impl World {
    pub fn new(size: usize, seed: u32) -> Self {
        let mut generator = WorldGenerator::new(seed, size);
        generator.generate()
    }

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

    pub fn update(&mut self, player_pos: (usize, usize)) {
        self.update_visibility(player_pos, FOG_RADIUS);
    }

    fn update_visibility(&mut self, (px, py): (usize, usize), radius: i32) {
        let (px, py) = (px as i32, py as i32);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
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
                        let dist = dx.abs() + dy.abs();

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