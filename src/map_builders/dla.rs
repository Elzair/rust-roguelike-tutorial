use rltk::{ line2d, LineAlg, Point, RandomNumberGenerator };
use specs::prelude::*;
use std::collections::HashMap;

use super::super::components::Position;
use super::super::map::{Map, TileType};
use super::super::spawner;
use super::super::SHOW_MAPGEN_VISUALIZER;
use super::common;
use super::MapBuilder;

#[derive(Clone, Copy, PartialEq)]
pub enum DLAAlgorithm {
    CentralAttractor,
    WalkInwards,
    WalkOutwards,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DLASymmetry {
    Both,
    Horizontal,
    None,
    Vertical,
}

pub struct DLASettings {
    pub algorithm: DLAAlgorithm,
    pub brush_size: i32,
    pub floor_percent: f32,
    pub symmetry: DLASymmetry,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DLASettings,
}

impl DLABuilder {
    #[allow(dead_code)]
    pub fn new(new_depth: i32, settings: DLASettings) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings,
        }
    }

    fn apply_paint(&mut self, x: i32, y: i32) {
        match self.settings.brush_size {
            1 => {
                let digger_idx = self.map.xy_idx(x, y).unwrap();
                self.map.tiles[digger_idx] = TileType::Floor;
            }
    
            _ => {
                let half_brush_size = self.settings.brush_size/2;
                for brush_y in y-half_brush_size..y+half_brush_size {
                    for brush_x in x-half_brush_size..x+half_brush_size {
                        if brush_x > 1 && brush_x < self.map.width-1 
                                && brush_y > 1 && brush_y < self.map.height-1 {
                            let idx = self.map.xy_idx(brush_x, brush_y).unwrap();
                            self.map.tiles[idx] = TileType::Floor;
                        }
                    }
                }
            }
        }
    }

    pub fn central_attractor(new_depth : i32) -> DLABuilder {
        DLABuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings: DLASettings {
                algorithm: DLAAlgorithm::CentralAttractor,
                brush_size: 2,
                floor_percent: 0.25,
                symmetry: DLASymmetry::None,
            },
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        const SNAPSHOT_INTERVAL: usize = 50;

        let mut rng = RandomNumberGenerator::new();

        // Set a central starting point
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y)
            .unwrap();
        self.take_snapshot();
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx-1] = TileType::Floor;
        self.map.tiles[start_idx+1] = TileType::Floor;
        self.map.tiles[start_idx-self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx+self.map.width as usize] = TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        
        let mut i: usize = 0; 

        while floor_tile_count  < desired_floor_tiles {
            match self.settings.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, self.map.width-3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height-3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y).unwrap();
                    while self.map.tiles[digger_idx] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < self.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -= 1; } }
                            _ => { if digger_y < self.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = self.map.xy_idx(digger_x, digger_y).unwrap();
                    }
                    self.paint(prev_x, prev_y);
                }
                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = self.starting_position.x;
                    let mut digger_y = self.starting_position.y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y).unwrap();
                    while self.map.tiles[digger_idx] == TileType::Floor {
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < self.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -=1; } }
                            _ => { if digger_y < self.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = self.map.xy_idx(digger_x, digger_y).unwrap();
                    }
                    self.paint(digger_x, digger_y);
                }
                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y).unwrap();
                
                    let mut path = line2d(
                        LineAlg::Bresenham, 
                        Point::new( digger_x, digger_y ), 
                        Point::new( self.starting_position.x, self.starting_position.y )
                    );
                
                    while self.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_idx = self.map.xy_idx(digger_x, digger_y).unwrap();
                    }
                    self.paint(prev_x, prev_y);
                }
            }

            if i % SNAPSHOT_INTERVAL == 0 {
                self.take_snapshot();
            }
            i += 1;

            floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }
        self.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile =
            common::remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = common::generate_voronoi_spawn_regions(&self.map, &mut rng);
    }

    pub fn insectoid(new_depth : i32) -> DLABuilder {
        DLABuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings: DLASettings {
                algorithm: DLAAlgorithm::CentralAttractor,
                brush_size: 2,
                floor_percent: 0.25,
                symmetry: DLASymmetry::Horizontal,
            },
        }
    }

    fn paint(&mut self, x: i32, y: i32) {
        match self.settings.symmetry {
            DLASymmetry::None => self.apply_paint(x, y),
            DLASymmetry::Horizontal => {
                let center_x = self.map.width / 2;
                if x == center_x {
                    self.apply_paint(x, y);                    
                } else {
                    let dist_x = i32::abs(center_x - x);
                    self.apply_paint(center_x + dist_x, y);
                    self.apply_paint(center_x - dist_x, y);
                }
            }
            DLASymmetry::Vertical => {
                let center_y = self.map.height / 2;
                if y == center_y {
                    self.apply_paint(x, y);
                } else {
                    let dist_y = i32::abs(center_y - y);
                    self.apply_paint(x, center_y + dist_y);
                    self.apply_paint(x, center_y - dist_y);
                }
            }
            DLASymmetry::Both => {
                let center_x = self.map.width / 2;
                let center_y = self.map.height / 2;
                if x == center_x && y == center_y {
                    self.apply_paint(x, y);
                } else {
                    let dist_x = i32::abs(center_x - x);
                    self.apply_paint(center_x + dist_x, y);
                    self.apply_paint(center_x - dist_x, y);
                    let dist_y = i32::abs(center_y - y);
                    self.apply_paint(x, center_y + dist_y);
                    self.apply_paint(x, center_y - dist_y);
                }
            }
        }
    }

    pub fn walk_inwards(new_depth : i32) -> DLABuilder {
        DLABuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings: DLASettings {
                algorithm: DLAAlgorithm::WalkInwards,
                brush_size: 1,
                floor_percent: 0.25,
                symmetry: DLASymmetry::None,
            },
        }
    }

    pub fn walk_outwards(new_depth : i32) -> DLABuilder {
        DLABuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings: DLASettings {
                algorithm: DLAAlgorithm::WalkOutwards,
                brush_size: 2,
                floor_percent: 0.25,
                symmetry: DLASymmetry::None,
            },
        }
    }
}

impl MapBuilder for DLABuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}
