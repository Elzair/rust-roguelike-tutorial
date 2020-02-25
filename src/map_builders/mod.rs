use rltk::RandomNumberGenerator;
use specs::prelude::*;

use super::components::Position;
use super::map::Map;

mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod common;
mod dla;
use dla::DLABuilder;
mod drunkard;
use drunkard::DrunkardsWalkBuilder;
mod maze;
use maze::MazeBuilder;
mod prefab_builder;
use prefab_builder::PrefabBuilder;
mod simple_map;
use simple_map::SimpleMapBuilder;
mod voronoi;
use voronoi::VoronoiCellBuilder;
mod waveform_collapse;
use waveform_collapse::WaveformCollapseBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&mut self) -> Map;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn get_starting_position(&mut self) -> Position;
    fn spawn_entities(&mut self, ecs: &mut World);
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // let mut rng = RandomNumberGenerator::new();
    // let mut result: Box<dyn MapBuilder> = match rng.roll_dice(1, 16) {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     2 => Box::new(BspInteriorBuilder::new(new_depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(new_depth)),
    //     4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
    //     5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
    //     6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
    //     7 => Box::new(DrunkardsWalkBuilder::fat_passage(new_depth)),
    //     8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
    //     9 => Box::new(MazeBuilder::new(new_depth)),
    //     10 => Box::new(DLABuilder::central_attractor(new_depth)),
    //     11 => Box::new(DLABuilder::insectoid(new_depth)),
    //     12 => Box::new(DLABuilder::walk_inwards(new_depth)),
    //     13 => Box::new(DLABuilder::walk_outwards(new_depth)),
    //     14 => Box::new(VoronoiCellBuilder::manhattan(new_depth)),
    //     15 => Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth)),
    // };

    // if rng.roll_dice(1, 3) == 1 {
    //     result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    // }

    // result
    Box::new(PrefabBuilder::new(
        new_depth,
        Some(Box::new(CellularAutomataBuilder::new(new_depth))),
    ))
}
