use super::components::{Monster, Position, Viewshed};
use super::map::Map;
use rltk::{console, field_of_view, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = ( 
        ReadStorage<'a, Viewshed>,              
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>, 
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, pos, monster) = data;

        for (viewshed, pos, _monster) in (&viewshed, &pos, &monster).join() {
            console::log("Monster considers their own existence");
        }
    }
}
