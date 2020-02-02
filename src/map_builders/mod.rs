use specs::prelude::*;

use super::components::Position;
use super::map::Map;

mod common;
mod simple_map;
use simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // NOte that until we have a second map type, this is not even slightly random. 
    Box::new(SimpleMapBuilder::new(new_depth))
}
