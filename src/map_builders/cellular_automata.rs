use rltk::{ CellularDistanceFunction, DijkstraMap, FastNoise, RandomNumberGenerator };
use specs::prelude::*;
use std::collections::HashMap;

use super::super::components::Position;
use super::super::map::{Map, TileType};
use super::super::spawner;
use super::super::SHOW_MAPGEN_VISUALIZER;
use super::MapBuilder;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // First randomize the map (set ~55% to be wall)
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y).unwrap();
                self.map.tiles[idx] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                };
            }
        }
        self.take_snapshot();

        // Now iteratively apply cellular automata rules
        for _i in 0..15 {
            let mut newtiles = self.map.tiles.clone();

            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y).unwrap();
                    let mut neighbors = 0;
                    if self.map.tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        // Find a starting point; start at the middle random walk until finding an open tile
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y).unwrap();
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y).unwrap();
        }
        // let mut start_x = self.map.width / 2;
        // let mut start_y = self.map.height / 2;
        // let mut start_idx = self.map.xy_idx(start_x, start_y).unwrap();
        // while self.map.tiles[start_idx] != TileType::Floor {
        //     start_x += rng.roll_dice(1, 6) - 3;
        //     start_y += rng.roll_dice(1, 6) - 3;
        //     if let Some(idx) = self.map.xy_idx(start_x, start_y) {
        //         start_idx = idx;
        //     }
        // }

        // Find all tiles we can reach from the starting point
        let map_starts: Vec<usize> = vec![start_idx];
        let djikstra_map = DijkstraMap::new(
            self.map.width,
            self.map.height,
            &map_starts,
            &self.map,
            200.0,
        );
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = djikstra_map.map[i];
                // We cannot get to this tile - so we will make it a wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                } else {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = TileType::DownStairs;
        self.take_snapshot();

        // Build a noise map for spawning entities
        let mut noise = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);

        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let idx = self.map.xy_idx(x, y).unwrap();
                if self.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if self.noise_areas.contains_key(&cell_value) {
                        self.noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        self.noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }
    }
}

impl MapBuilder for CellularAutomataBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
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
