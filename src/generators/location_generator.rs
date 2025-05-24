use rand::{Rng, SeedableRng, rngs::StdRng};
use crate::systems::world::TerrainType;
use crate::systems::location::{Location, Species};
use crate::systems::position::Position;
use noise::NoiseFn;

#[derive(PartialEq)]
pub struct LocationMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<LocationTile>>,
    pub points_of_interest: Vec<PointOfInterest>,
}

impl LocationMap {
    pub fn find_spawn_position(&self) -> Position{
        let center_x = self.width / 2;
        let center_y = self.height / 2;
        
        // Try to find a walkable tile near the center
        for radius in 0..5 {
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let x = (center_x as i32 + dx).clamp(0, self.width as i32 - 1) as usize;
                    let y = (center_y as i32 + dy).clamp(0, self.height as i32 - 1) as usize;
                    
                    let tile = &self.tiles[y][x];
                    if self.is_walkable(x, y) {
                        return Position{ x, y};
                    }
                }
            }
        }
        
        // Fallback to the first walkable tile we can find
        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_walkable(x, y) {
                        return Position{ x, y};
                }
            }
        }
        
        // Ultimate fallback
        Position {x:1, y:1}
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        let tile = &self.tiles[y][x];
        !matches!(tile.tile_type, 
            LocationTileType::Wall | 
            LocationTileType::HumanHouse | 
            LocationTileType::ElfTreehouse | 
            LocationTileType::OrcHut
        )
    }
}


#[derive(Clone, PartialEq)]
pub struct LocationTile {
    pub blocked: bool,
    pub tile_type: LocationTileType,
    pub feature: Option<Feature>,
}

#[derive(Clone, PartialEq)]
pub enum LocationTileType {
    // Base tiles
    Ground,
    Wall,
    Water,
    
    // Species-specific tiles
    HumanRoad,
    ElfPath,
    OrcTrail,
    
    // Building types
    HumanHouse,
    ElfTreehouse,
    OrcHut,
    Trading,
    Shrine,
}

#[derive(Clone, PartialEq)]
pub struct Feature {
    pub name: String,
    pub feature_type: FeatureType,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FeatureType {
    Market,
    Temple,
    Tavern,
    Blacksmith,
    Garden,
    TrainingGround,
    Storage,
}
#[derive(Clone, PartialEq)]
pub struct PointOfInterest {
    pub position: Position,
    pub feature: Feature,
}

pub struct LocationGenerator {
    rng: StdRng,
    base_terrain: TerrainType,
    location: Location,
}

impl LocationGenerator {
    pub fn new(seed: u64, base_terrain: TerrainType, location: Location) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            base_terrain,
            location,
        }
    }

    pub fn generate(&mut self) -> LocationMap {
        let (width, height) = self.determine_map_size();
        let mut map = self.create_empty_map(width, height);
        
        match self.location.species {
            Species::Human => self.generate_human_settlement(&mut map),
            Species::Elf => self.generate_elf_settlement(&mut map),
            //Species::Orc => self.generate_orc_settlement(&mut map),
            // ... other species
            _ => self.generate_basic_settlement(&mut map),
        }

        map
    }

    fn determine_map_size(&mut self) -> (usize, usize) {
        // Base size on location.size, cast to i32 to handle negative values
        let base_size = (self.location.size as f32).sqrt() as i32;
        let size_variance = self.rng.gen_range(-2..=2);
        let size = (base_size + size_variance).max(5) as usize;  // Ensure minimum size and cast back to usize
        (size * 2, size * 2) // Make maps rectangular
    }

    fn generate_human_settlement(&mut self, map: &mut LocationMap) {
        // Generate road network
        self.generate_road_network(map);
        // Place central features (market, temple)
        self.place_central_features(map);
        // Add houses along roads
        self.place_houses(map);
        // Add defensive walls if size > threshold
        if self.location.size > 50 {
            self.add_walls(map);
        }
    }

    fn generate_elf_settlement(&mut self, map: &mut LocationMap) {
        // Create organic paths
        self.generate_natural_paths(map);
        // Place treehouses in clusters
        self.place_treehouses(map);
        // Add gardens and shrines
        self.place_nature_features(map);
    }

    fn generate_basic_settlement(&mut self, map: &mut LocationMap) {
        // Basic settlement generation
        self.generate_road_network(map);
        self.place_central_features(map);
    }

    fn create_empty_map(&self, width: usize, height: usize) -> LocationMap {
        let tiles = vec![vec![LocationTile {
            blocked: false,
            tile_type: LocationTileType::Ground,
            feature: None,
        }; width]; height];

        LocationMap {
            width,
            height,
            tiles,
            points_of_interest: Vec::new(),
        }
    }

    fn generate_road_network(&mut self, map: &mut LocationMap) {
        // Create a main road from east to west
        let mid_y = map.height / 2;
        for x in 0..map.width {
            map.tiles[mid_y][x].tile_type = LocationTileType::HumanRoad;
        }

        // Create a main road from north to south
        let mid_x = map.width / 2;
        for y in 0..map.height {
            map.tiles[y][mid_x].tile_type = LocationTileType::HumanRoad;
        }

        // Add some random branch roads
        let num_branches = self.rng.gen_range(2..=4);
        for _ in 0..num_branches {
            let start_x = self.rng.gen_range(0..map.width);
            let start_y = if self.rng.gen_bool(0.5) { 0 } else { map.height - 1 };
            self.create_branching_road(map, (start_x, start_y));
        }
    }

    fn create_branching_road(&mut self, map: &mut LocationMap, start: (usize, usize)) {
        let target_x = map.width / 2;
        let target_y = map.height / 2;
        let mut current = start;

        while current.0 != target_x || current.1 != target_y {
            map.tiles[current.1][current.0].tile_type = LocationTileType::HumanRoad;
            
            // Move towards center with some randomness
            if self.rng.gen_bool(0.7) {
                if current.0 < target_x { current.0 += 1; }
                else if current.0 > target_x { current.0 -= 1; }
            } else {
                if current.1 < target_y { current.1 += 1; }
                else if current.1 > target_y { current.1 -= 1; }
            }
        }
    }

    // Update methods that call place_feature
    fn place_central_features(&mut self, map: &mut LocationMap) {
        let center_x = map.width / 2;
        let center_y = map.height / 2;

        // Place market in center
        self.place_feature(map, Position { x: center_x, y: center_y }, FeatureType::Market);

        // Place temple nearby
        let temple_x = (center_x as i32 + self.rng.gen_range(-2..=2))
            .clamp(0, map.width as i32 - 1) as usize;
        let temple_y = (center_y as i32 + self.rng.gen_range(-2..=2))
            .clamp(0, map.height as i32 - 1) as usize;
        self.place_feature(map, Position { x: temple_x, y: temple_y }, FeatureType::Temple);

        // Place tavern
        let tavern_x = (center_x as i32 + self.rng.gen_range(-3..=3))
            .clamp(0, map.width as i32 - 1) as usize;
        let tavern_y = (center_y as i32 + self.rng.gen_range(-3..=3))
            .clamp(0, map.height as i32 - 1) as usize;
        self.place_feature(map, Position { x: tavern_x, y: tavern_y }, FeatureType::Tavern);
    }

    // Update place_houses method
    fn place_houses(&mut self, map: &mut LocationMap) {
        for y in 1..map.height-1 {
            for x in 1..map.width-1 {
                if map.tiles[y][x].tile_type == LocationTileType::Ground {
                    let near_road = self.is_adjacent_to_road(map, x, y);
                    if near_road && self.rng.gen_bool(0.3) {
                        map.tiles[y][x].tile_type = LocationTileType::HumanHouse;
                        
                        if self.rng.gen_bool(0.1) {
                            let feature_type = if self.rng.gen_bool(0.5) {
                                FeatureType::Blacksmith
                            } else {
                                FeatureType::Storage
                            };
                            self.place_feature(map, Position { x, y }, feature_type);
                        }
                    }
                }
            }
        }
    }

    fn add_walls(&mut self, map: &mut LocationMap) {
        // First pass: place walls around the settlement perimeter
        let wall_distance = 3; // Distance from edge of map
        
        // Top and bottom walls
        for x in wall_distance..map.width-wall_distance {
            map.tiles[wall_distance][x].tile_type = LocationTileType::Wall;
            map.tiles[map.height-wall_distance-1][x].tile_type = LocationTileType::Wall;
        }
        
        // Left and right walls
        for y in wall_distance..map.height-wall_distance {
            map.tiles[y][wall_distance].tile_type = LocationTileType::Wall;
            map.tiles[y][map.width-wall_distance-1].tile_type = LocationTileType::Wall;
        }

        // Add gates at road intersections
        for x in 0..map.width {
            if map.tiles[map.height/2][x].tile_type == LocationTileType::HumanRoad {
                map.tiles[wall_distance][x].tile_type = LocationTileType::HumanRoad;
                map.tiles[map.height-wall_distance-1][x].tile_type = LocationTileType::HumanRoad;
            }
        }
        for y in 0..map.height {
            if map.tiles[y][map.width/2].tile_type == LocationTileType::HumanRoad {
                map.tiles[y][wall_distance].tile_type = LocationTileType::HumanRoad;
                map.tiles[y][map.width-wall_distance-1].tile_type = LocationTileType::HumanRoad;
            }
        }
    }

    fn is_adjacent_to_road(&self, map: &LocationMap, x: usize, y: usize) -> bool {
        let directions = [
            (0, 1), (1, 0), (0, -1), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ];

        for (dx, dy) in directions.iter() {
            let new_x = (x as i32 + dx).clamp(0, map.width as i32 - 1) as usize;
            let new_y = (y as i32 + dy).clamp(0, map.height as i32 - 1) as usize;
            
            if map.tiles[new_y][new_x].tile_type == LocationTileType::HumanRoad {
                return true;
            }
        }
        false
    }
    fn generate_natural_paths(&mut self, map: &mut LocationMap) {
        // Generate organic, curvy paths using noise
        let noise = noise::OpenSimplex::new(self.rng.gen_range(0..=u32::MAX));
        
        for y in 0..map.height {
            for x in 0..map.width {
                let value = noise.get([x as f64 * 0.1, y as f64 * 0.1]);
                if value > 0.7 {
                    map.tiles[y][x].tile_type = LocationTileType::ElfPath;
                }
            }
        }
    }

    fn place_treehouses(&mut self, map: &mut LocationMap) {
        let num_clusters = self.rng.gen_range(3..=5);
        
        for _ in 0..num_clusters {
            let center_x = self.rng.gen_range(5..map.width-5);
            let center_y = self.rng.gen_range(5..map.height-5);
            
            // Create a cluster of 3-6 treehouses
            let num_houses = self.rng.gen_range(3..=6);
            for _ in 0..num_houses {
                let offset_x = self.rng.gen_range(-2..=2);
                let offset_y = self.rng.gen_range(-2..=2);
                let pos_x = (center_x as i32 + offset_x).clamp(0, map.width as i32 - 1) as usize;
                let pos_y = (center_y as i32 + offset_y).clamp(0, map.height as i32 - 1) as usize;
                
                map.tiles[pos_y][pos_x].tile_type = LocationTileType::ElfTreehouse;
            }
        }
    }

    fn place_nature_features(&mut self, map: &mut LocationMap) {
        // Place gardens
        let num_gardens = self.rng.gen_range(3..=6);
        for _ in 0..num_gardens {
            // Generate coordinates first
            let x = self.rng.gen_range(0..map.width);
            let y = self.rng.gen_range(0..map.height);
            // Then create Position and place feature
            self.place_feature(map, Position { x, y }, FeatureType::Garden);
        }

        // Place shrines in quiet corners
        let num_shrines = self.rng.gen_range(2..=4);
        for _ in 0..num_shrines {
            let x = self.rng.gen_range(0..map.width);
            let y = self.rng.gen_range(0..map.height);
            if !self.is_near_feature(map, Position{x, y}, 5) {
                self.place_feature(map, Position{x, y}, FeatureType::Temple);
            }
        }
    }

    fn place_feature(&mut self, map: &mut LocationMap, pos: Position, feature_type: FeatureType) {
        if pos.x < map.width && pos.y < map.height {
            let feature = Feature {
                name: self.get_feature_name(feature_type),
                feature_type,
            };
            map.tiles[pos.y][pos.x].feature = Some(feature.clone());
            map.points_of_interest.push(PointOfInterest {
                position: pos,
                feature,
            });
        }
    }

       // Update is_near_feature to use Position
    fn is_near_feature(&self, map: &LocationMap, pos: Position, distance: i32) -> bool {
        for poi in &map.points_of_interest {
            let dx = (pos.x as i32 - poi.position.x as i32).abs();
            let dy = (pos.y as i32 - poi.position.y as i32).abs();
            if dx <= distance && dy <= distance {
                return true;
            }
        }
        false
    }

    fn get_feature_name(&mut self, feature_type: FeatureType) -> String {
        match feature_type {
            FeatureType::Market => "Town Market".to_string(),
            FeatureType::Temple => "Sacred Temple".to_string(),
            FeatureType::Tavern => "The Wanderer's Rest".to_string(),
            FeatureType::Garden => "Natural Garden".to_string(),
            _ => "Feature".to_string(),
        }
    }
}