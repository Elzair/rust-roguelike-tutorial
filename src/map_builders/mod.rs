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
mod drunkard;
use drunkard::{DrunkSpawnMode, DrunkardSettings, DrunkardsWalkBuilder};
mod simple_map;
use simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&mut self) -> Map;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn get_starting_position(&mut self) -> Position;
    fn spawn_entities(&mut self, ecs: &mut World);
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = RandomNumberGenerator::new();
    // match rng.roll_dice(1, 4) {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     2 => Box::new(BspInteriorBuilder::new(new_depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth)),
    // }
    Box::new(DrunkardsWalkBuilder::new(
        new_depth,
        DrunkardSettings {
            drunken_lifetime: 100,
            spawn_mode: DrunkSpawnMode::Random,
        },
    ))
}
