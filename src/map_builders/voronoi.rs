use rltk::{ DistanceAlg, Point, RandomNumberGenerator };
use specs::prelude::*;
use std::collections::HashMap;

use super::super::components::Position;
use super::super::map::{Map, TileType};
use super::super::spawner;
use super::super::SHOW_MAPGEN_VISUALIZER;
use super::common::{ self, Symmetry };
use super::MapBuilder;

pub struct VoronoiBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl VoronoiBuilder {
    #[allow(dead_code)]
    pub fn new(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        const SNAPSHOT_INTERVAL: usize = 50;

        let mut rng = RandomNumberGenerator::new();

        // Generate Voronoi Diagram
        // First generate `n_seeds` randomly distributed about the map
        let n_seeds = 64;
        let mut voronoi_seeds: Vec<(usize, Point)> = Vec::new();

        while voronoi_seeds.len() < n_seeds {
            let vx = rng.roll_dice(1, self.map.width-1);
            let vy = rng.roll_dice(1, self.map.height-1);
            let vidx = self.map.xy_idx(vx, vy).unwrap();
            let candidate = (vidx, Point::new(vx, vy));

            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        // Determine each cell's membership by determining the closest seed to it
        let mut voronoi_distance = vec![(0, 0.0f32); n_seeds];
        let mut voronoi_membership: Vec<i32> = vec![0; self.map.width as usize * self.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % self.map.width;
            let y = i as i32 / self.map.width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance = DistanceAlg::PythagorasSquared.distance2d(
                    Point::new(x, y), 
                    pos.1
                );
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voronoi_distance[0].0 as i32;
        }

        // Place floors only on cells that border 0 or 1 other Voronoi groups
        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let mut neighbors = 0;
                let my_idx = self.map.xy_idx(x, y).unwrap();
                let my_seed = voronoi_membership[my_idx];

                if voronoi_membership[self.map.xy_idx(x-1, y).unwrap()] != my_seed { neighbors += 1; }
                if voronoi_membership[self.map.xy_idx(x+1, y).unwrap()] != my_seed { neighbors += 1; }
                if voronoi_membership[self.map.xy_idx(x, y-1).unwrap()] != my_seed { neighbors += 1; }
                if voronoi_membership[self.map.xy_idx(x, y+1).unwrap()] != my_seed { neighbors += 1; }

                if neighbors <  2 {
                    self.map.tiles[my_idx] = TileType::Floor;
                }
            }

            self.take_snapshot();
        }

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

impl MapBuilder for VoronoiBuilder {
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
