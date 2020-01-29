use specs::prelude::*;

use super::components::{
    EntityMoved, EntryTrigger, Hidden, InflictsDamage, Name, Position, SingleActivation,
    SufferDamage,
};
use super::gamelog::GameLog;
use super::map::Map;
use super::particle_system::ParticleBuilder;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, InflictsDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, SingleActivation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            mut hidden,
            names,
            entities,
            mut log,
            inflicts_damage,
            mut particle_builder,
            mut suffer_damage,
            single_activation,
        ) = data;

        // Iterate the entities that moved and their final position
        let mut remove_entities: Vec<Entity> = Vec::new();

        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y).unwrap();

            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id {
                    // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(*entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            // TODO: Log name of trap.
                            // Something triggered it
                            let name = names.get(*entity_id);
                            if let Some(name) = name {
                                log.entries.insert(0, format!("{} triggers!", &name.name));
                            }

                            hidden.remove(*entity_id); // The trap is no longer hidden

                            // If the trap does damage, inflict it.
                            let damage = inflicts_damage.get(*entity_id);
                            if let Some(damage) = damage {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                                suffer_damage
                                    .insert(
                                        entity,
                                        SufferDamage {
                                            amount: damage.damage,
                                        },
                                    )
                                    .expect("Unable to do damage");

                                // If it is a single activation, remove it.
                                let sa = single_activation.get(*entity_id);
                                if let Some(_sa) = sa {
                                    remove_entities.push(*entity_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove any single activation traps
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
