use rltk::{ console, RandomNumberGenerator };
use specs::prelude::*;
use std::collections::HashMap;

use super::super::components::Position;
use super::super::map::{Map, TileType};
use super::super::spawner;
use super::super::SHOW_MAPGEN_VISUALIZER;
use super::common;
use super::MapBuilder;

#[derive(Clone, Copy, PartialEq)]
pub enum DrunkSpawnMode {
    Random,
    StartingPoint,
}

pub struct DrunkardSettings {
    pub drunken_lifetime: i32,
    pub spawn_mode: DrunkSpawnMode,
}

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DrunkardSettings,
}

impl DrunkardsWalkBuilder {
    pub fn new(new_depth: i32, settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
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
        self.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (total_tiles / 2) as usize;
        let mut floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        let mut digger_count = 0;
        let mut active_digger_count= 0;

        while floor_tile_count < desired_floor_tiles {
            let mut did_something= false;
            // let mut drunk_x = self.starting_position.x;
            // let mut drunk_y = self.starting_position.y;
            let (mut drunk_x, mut drunk_y) = match self.settings.spawn_mode {
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        (self.starting_position.x, self.starting_position.y)
                    } else {
                        (rng.roll_dice(1, self.map.width-3)+1, rng.roll_dice(1, self.map.height-3)+1)
                    }
                }
                DrunkSpawnMode::StartingPoint => (self.starting_position.x, self.starting_position.y),
            };
            let mut drunk_life = self.settings.drunken_lifetime;

            while drunk_life > 0 {
                let drunk_idx = self.map.xy_idx(drunk_x, drunk_y).unwrap();
                if self.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                self.map.tiles[drunk_idx] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => { if drunk_x > 2 { drunk_x -= 1; } }
                    2 => { if drunk_x < self.map.width-2 { drunk_x += 1; } }
                    3 => { if drunk_y > 2 { drunk_y -= 1; } }
                    _ => { if drunk_y < self.map.height-2 { drunk_y += 1; } }
                }

                drunk_life -= 1;
            }

            if did_something {
                self.take_snapshot();
                active_digger_count += 1;
            }
        
            digger_count += 1;
            for t in self.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }
        console::log(
            format!(
                "{} dwarves gave up their sobriety, of whom {} actually found a wall.",
                digger_count,
                active_digger_count
            )
        );

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
}

impl MapBuilder for DrunkardsWalkBuilder {
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
