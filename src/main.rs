rltk::add_wasm_support!();
use rltk::{Console, GameState, Point, Rltk};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
#[macro_use]
extern crate specs_derive;

mod components;
pub use components::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InBackpack, InflictsDamage, Item, 
    Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializationHelper, 
    SerializeMe, SufferDamage, Viewshed, WantsToDropItem, WantsToMelee, WantsToPickupItem, 
    WantsToUseItem,
};
mod damage_system;
pub use damage_system::DamageSystem;
mod gamelog;
mod gui;
mod inventory_system;
pub use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};
mod map;
pub use map::*;
mod map_indexing_system;
pub use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
pub use melee_combat_system::MeleeCombatSystem;
mod monster_ai_system;
pub use monster_ai_system::MonsterAI;
mod player;
pub use player::*;
mod rect;
pub use rect::*;
mod saveload_system;
mod spawner;
mod visibility_system;
pub use visibility_system::VisibilitySystem;

#[derive(Clone, Copy, PartialEq)]
pub enum RunState {
    AwaitingInput,
    MainMenu { menu_selection: gui::MainMenuSelection },
    MonsterTurn,
    PlayerTurn,
    PreRun,
    SaveGame,
    ShowDropItem,
    ShowInventory,
    ShowTargeting { range: i32, item: Entity },
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut items = ItemUseSystem {};
        items.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate: RunState;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();

        match newrunstate {
            RunState::MainMenu{..} => {}
            _ => {
                draw_map(&self.ecs, ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y).unwrap();
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match newrunstate {
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::MainMenu{ .. } => {
                use gui::MainMenuResult::*;
                use gui::MainMenuSelection::*;
                let result = gui::main_menu(self, ctx);
                match result {
                    NoSelection{ selected } => {
                        newrunstate = RunState::MainMenu{ menu_selection: selected }
                    }
                    Selected{ selected } => {
                        match selected {
                            NewGame => newrunstate = RunState::PreRun,
                            LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            },
                            Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu{ menu_selection: gui::MainMenuSelection::LoadGame };
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem { item: item_entity, target: None },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowTargeting{range, item} => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: result.1 }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);
    }
}

fn main() {
    let mut context = Rltk::init_simple8x8(80, 50, "Hello Rust World", "resources");
    context.with_post_scanlines(true);
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map = Map::new_map_rooms_and_corridors(1);
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    // Populate rooms with monsters
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame });
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    rltk::main_loop(context, gs);
}
