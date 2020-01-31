use super::map::Map;

mod common;
mod simple_map;
use simple_map::SimpleMapBuilder;

trait MapBuilder {
    fn build(new_depth: i32) -> Map;
}

pub fn build_random_map(new_depth: i32) -> Map {
    SimpleMapBuilder::build(new_depth)
}