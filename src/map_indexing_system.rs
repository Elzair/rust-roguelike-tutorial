use specs::prelude::*;
use super::components::{BlocksTile, Position};
use super::map::Map;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a>, );
    
    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            // If they block, update the blocking list
            // let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_) = blockers.get(entity) { map.blocked[idx] = true; }

            // Push the entity to the appropriate index slot. It is a Copy 
            // type, so we do not need to clone it (we want to avoid moving
            // it out of the ECS!)
            map.tile_content[idx].push(entity);
        }
    }
}