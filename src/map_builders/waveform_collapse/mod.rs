use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

use super::{common as mbcommon, MapBuilder};
use super::super::components::Position;
use super::super::map::{Map, TileType};
use super::super::{spawner, SHOW_MAPGEN_VISUALIZER};

mod common;
use common::MapChunk;
mod constraints;
mod image_loader;
mod solver;
use solver::Solver;

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    derive_from: Box<dyn MapBuilder>,
}

impl WaveformCollapseBuilder {
    #[allow(dead_code)]
    pub fn new(new_depth: i32, derive_from: Box<dyn MapBuilder>) -> Self {
        WaveformCollapseBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            derive_from
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        const CHUNK_SIZE: i32 = 8;

        // Prebuild map to get chunks
        let prebuilder = &mut self.derive_from.as_mut();
        prebuilder.build_map();
        self.map = prebuilder.get_map();

        // Remove any stairs from prebuilt map since we will place them
        for t in self.map.tiles.iter_mut() {
            if *t == TileType::DownStairs { *t = TileType::Floor; }
        }
        self.take_snapshot();

        let patterns = constraints::build_patterns(&self.map, CHUNK_SIZE, true, true);
        let constraints = constraints::patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE);

        self.map = Map::new(self.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, &mut rng) {
                self.take_snapshot();
            }
            self.take_snapshot();
            if solver.possible { break; } // If it has hit an impossible condition, try again
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let mut start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y)
            .unwrap();
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y).unwrap();
        }
        self.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile =
            mbcommon::remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = mbcommon::generate_voronoi_spawn_regions(&self.map, &mut rng);
    }

    pub fn derived_map(new_depth: i32, builder: Box<dyn MapBuilder>) -> Self {
        WaveformCollapseBuilder::new(new_depth, builder)
    }

    fn render_tile_gallery(&mut self, constraints: &Vec<MapChunk>, chunk_size: i32) {
        self.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            constraints::render_pattern_to_map(&mut self.map, &constraints[counter], chunk_size, x, y);
    
            x += chunk_size + 1;
            if x + chunk_size > self.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;
    
                if y + chunk_size > self.map.height {
                    // Move to the next page
                    self.take_snapshot();
                    self.map = Map::new(0);
    
                    x = 1;
                    y = 1;
                }
            }
    
            counter += 1;
        }
        self.take_snapshot();
    }
}

impl MapBuilder for WaveformCollapseBuilder {
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
