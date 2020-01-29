use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::error::NoError;
use specs::saveload::{ConvertSaveload, Marker};
use rltk::RGB;

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct BlocksTile {}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct Consumable {}

#[derive(Clone, Component, ConvertSaveload)]
pub struct DefenseBonus {
    pub defense: i32, 
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct EntityMoved {}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct EntryTrigger {}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
pub enum EquipmentSlot { Melee, Shield, }

#[derive(Clone, Component, Deserialize, Serialize)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Clone, Component, ConvertSaveload)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct Hidden {}

#[derive(Clone, Component, Deserialize, Serialize)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
pub enum HungerState {
    Hungry,
    Normal,
    Starving,
    WellFed,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct Item {}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct MagicMapper {}

#[derive(Clone, Component, ConvertSaveload)]
pub struct MeleePowerBonus {
    pub power: i32,
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct Monster {}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Clone, Component, Deserialize, Serialize)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}

#[derive(Clone, Component, Deserialize, Serialize)]
pub struct Player {}

#[derive(Clone, Component, ConvertSaveload)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct ProvidesFood {}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Clone, Component, ConvertSaveload)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct SingleActivation {}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct SufferDamage {
    pub amount: i32,
}

#[derive(Clone, Component, ConvertSaveload)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct WantsToRemoveItem {
    pub item: Entity,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

pub struct SerializeMe;

// Special component that exists to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map : super::map::Map
}

// // Wrapper for Equipped
// #[derive(Clone, Deserialize, Serialize)]
// pub struct EquippedData<M>(M, EquipmentSlot);

// impl<M: Marker + Serialize> ConvertSaveload<M> for Equipped
// where
//     for<'de> M: Deserialize<'de>,
// {
//     type Data = EquippedData<M>;
//     type Error = NoError;

//     fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
//     where
//         F: FnMut(Entity) -> Option<M>,
//     {
//         let marker = ids(self.owner).unwrap();
//         Ok(EquippedData(marker, self.slot))
//     }

//     fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
//     where
//         F: FnMut(M) -> Option<Entity>,
//     {
//         let entity = ids(data.0).unwrap();
//         Ok(Equipped(owner: entity, slot: data.1))
//     }
// }