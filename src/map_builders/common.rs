use std::cmp::{ max, min };

use super::super::map::{Map, TileType};
use super::super::rect::Rect;

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            if let Some(idx) = map.xy_idx(x, y) {
                map.tiles[idx] = TileType::Floor;
            }
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        if let Some(idx) = map.xy_idx(x, y) {
            if idx > 0 {
                map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        if let Some(idx) = map.xy_idx(x, y) {
            if idx > 0 {
                map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
}