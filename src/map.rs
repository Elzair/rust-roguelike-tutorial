use rltk::{Algorithm2D, BaseMap, Console, Point, Rltk, RGB};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::collections::HashSet;

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TileType {
    DownStairs,
    Floor,
    Wall,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn new(new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth,
            bloodstains: HashSet::new(),
        }
    }


    // pub fn xy_idx(&self, x: i32, y: i32) -> Option<usize> {
    //     let idx = (y as usize * self.width as usize) + x as usize;
    //     if idx < self.width as usize * self.height as usize { Some(idx) }
    //     else { None }
    // }
    pub fn xy_idx(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height { 
            Some((y as usize * self.width as usize) + x as usize) 
        }
        else { None }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y).unwrap();
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
    
    fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx:usize) -> Vec<(usize, f32)> {
        let mut exits : Vec<(usize, f32)> = Vec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
    }
}

pub fn draw_map(map: &Map, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon the tile type
        if map.revealed_tiles[idx] {
            let (glyph, mut fg) = match tile {
                TileType::DownStairs => (rltk::to_cp437('>'), RGB::from_f32(0.0, 1.0, 1.0)),
                TileType::Floor => (rltk::to_cp437('.'), RGB::from_f32(0.0, 0.5, 0.5)),
                TileType::Wall => (wall_glyph(&*map, x, y), RGB::from_f32(0.0, 1.0, 0.)),
            };

            // Render bloodstain, if present
            let bg = if map.bloodstains.contains(&idx) { 
                RGB::from_f32(0.75, 0.0, 0.0) 
            } else { 
                RGB::from_f32(0.0, 0.0, 0.0) 
            };

            if !map.visible_tiles[idx] { fg = fg.to_greyscale() }
            ctx.set(x, y, fg, bg, glyph);
        }

        // Move the coordinates
        x += 1;
        if x > MAPWIDTH as i32 - 1 {
            x = 0;
            y += 1;
        }
    }
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> u8 {
    if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y-1) { mask += 1; }
    if is_revealed_and_wall(map, x, y+1) { mask += 2; }
    if is_revealed_and_wall(map, x-1, y) { mask += 4; }
    if is_revealed_and_wall(map, x+1, y) { mask += 8; }

    match mask {
        0 => 9,     // Pillar because we cannot see neighbors
        1 => 186,   // Wall to north
        2 => 186,   // Wall to south
        3 => 186,   // Wall to north and south
        4 => 205,   // Wall to west
        5 => 188,   // Wall to north and west
        6 => 187,   // Wall to south and west
        7 => 185,   // Wall to north, south, and west
        8 => 205,   // Wall to east
        9 => 200,   // Wall to north and east
        10 => 201,  // Wall to south and east
        11 => 204,  // Wall to north, south, and east
        12 => 205,  // Wall to east and west
        13 => 202,  // Wall to east, west, and south
        14 => 203,  // Wall to east, west, and north
        _ => 35     // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y).unwrap();
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
