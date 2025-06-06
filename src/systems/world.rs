use crate::generators::world_generator::WorldGenerator;
use crate::systems::location::{Location, Species};
use crate::systems::position::Position;
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
    Road,
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
            TerrainType::Road => '#',
            
        }
    }
}

pub type TileGrid = Vec<Vec<Tile>>;
const FOG_RADIUS: i32 = 4;

impl World {
    pub fn new(seed: u32, width:usize, height:usize) -> Self {
        let mut generator = WorldGenerator::new(seed, width, height);
        generator.generate()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_tile(&self, position: &Position) -> &Tile {
        &self.tiles[position.y][position.x]
    }

    pub fn get_wrapped_coordinates(&self, position: &Position) -> Position {
        if self.wraparound {
            Position {
                x: ((position.x as i32).rem_euclid(self.width() as i32)) as usize,
                y: ((position.y as i32).rem_euclid(self.height() as i32)) as usize,
            }
        } else {
            Position {
                x: position.x.clamp(0, self.width() - 1),
                y: position.y.clamp(0, self.height() - 1),
            }
        }
    }

    pub fn update(&mut self, player_pos: &Position) {
        self.update_visibility(player_pos, FOG_RADIUS);
    }

    fn update_visibility(&mut self, position: &Position, radius: i32) {
        let (px, py) = (position.x as i32, position.y as i32);
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                // Only update tiles within the circular radius
                if dx*dx + dy*dy <= radius*radius {
                    let world_x = px + dx;
                    let world_y = py + dy;
                    
                    let current_pos = Position {
                        x: world_x as usize,
                        y: world_y as usize,
                    };
                    
                    // Get the wrapped position and mark it as seen
                    if let Some(wrapped_pos) = self.get_valid_position(&current_pos) {
                        self.tiles[wrapped_pos.y][wrapped_pos.x].seen = true;
                    }
                }
            }
        }
    }

    // Helper function to get valid wrapped positions
    fn get_valid_position(&self, pos: &Position) -> Option<Position> {
        if self.wraparound {
            Some(Position {
                x: pos.x.rem_euclid(self.width),
                y: pos.y.rem_euclid(self.height),
            })
        } else if pos.x < self.width && pos.y < self.height {
            Some(*pos)
        } else {
            None
        }
    }

    pub fn find_nearest_species(
        &self,
        start: &Position,
        target_species: Species,
    ) -> Option<Position> {
        let (sx, sy) = (start.x as i32, start.y as i32);
        let mut closest: Option<(Position, i32)> = None;

        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if let Some(location) = &tile.location {
                    if location.species == target_species {
                        let dx = sx - x as i32;
                        let dy = sy - y as i32;
                        let dist = dx.abs() + dy.abs();

                        match closest {
                            Some((_, best_dist)) if dist < best_dist => {
                                closest = Some((Position { x, y }, dist));
                            }
                            None => {
                                closest = Some((Position { x, y }, dist));
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