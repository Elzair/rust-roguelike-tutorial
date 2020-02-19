use rltk::{console, RandomNumberGenerator};
use specs::prelude::*;

use super::super::{
    components::Position, map::Map, map::TileType, spawner, SHOW_MAPGEN_VISUALIZER,
};
use super::{common, MapBuilder};

mod prefab_level;

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub enum PrefabMode {
    Constant { level: prefab_level::PrefabLevel },
    RexLevel { template: &'static str },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
}

impl PrefabBuilder {
    pub fn new(new_depth: i32) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Constant {
                level: prefab_level::WFC_POPULATED,
            },
            // mode: PrefabMode::RexLevel {
            //     template: "../resources/wfc-populated.xp"
            // },
            spawns: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
            _ => {}
        }
        self.take_snapshot();

        // Find a starting point; start at the middle and walk left until finding an open tile
        if self.starting_position.x == 0 {
            self.starting_position = Position { x: 2, y: 2 };
            let mut start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y)
                .unwrap();
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self
                    .map
                    .xy_idx(self.starting_position.x, self.starting_position.y)
                    .unwrap();
            }
        }
        self.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                self.starting_position = Position {
                    x: x as i32,
                    y: y as i32,
                };
            }
            '>' => self.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Health Potion".to_string()));
            }
            _ => console::log(format!("Unknown glyph loading map: {}", ch)),
        }
    }

    #[allow(dead_code)]
    fn load_ascii_map(&mut self, level: &prefab_level::PrefabLevel) {
        // Start by converting to a vector, with newlines removed
        let mut string_vec: Vec<char> = level
            .template
            .chars()
            .filter(|a| *a != '\r' && *a != '\n')
            .collect();
        for c in string_vec.iter_mut() {
            if *c as u8 == 160u8 {
                *c = ' ';
            }
        }

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize && i < string_vec.len() {
                    let idx = self.map.xy_idx(tx as i32, ty as i32).unwrap();
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32).unwrap();
                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }
}

impl MapBuilder for PrefabBuilder {
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
        for entity in self.spawns.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
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
