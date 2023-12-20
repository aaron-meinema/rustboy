use sdl2::pixels::Color;
use std::fmt::Debug;
pub const WIDTH:  i32 = 160;
pub const HEIGHT: i32 = 144;
pub const DEBUG: i32 = 30;

pub struct Renderer {
    tile_data: [u8; 0x9fff - 0x8000],
    lcdc: u8,
    color: [Color; 4],
    transparent: Color,
}

pub struct ColorPosition {
    pub y: i32,
    pub x: i32,

    pub color: Color
}

impl Debug for ColorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorPosition").field("y", &self.y).field("x", &self.x).field("color", &self.color).finish()
    }
}


struct Tile {
    pixels: [[Color; 8]; 8],
}

impl Clone for Tile {
    fn clone(&self) -> Self {
        Self { pixels: self.pixels.clone() }
    }
}

impl Copy for Tile {}

impl Tile {
    pub fn new() -> Self {
        let color = Color::WHITE;
        Tile { pixels:[[color; 8]; 8] }
    }
}

impl Renderer {
    pub fn new() -> Self {
        let tile:[u8; 0x9fff - 0x8000]  = [0; 0x9fff - 0x8000];
        let renderer = Renderer {
            tile_data: tile,
            lcdc: 0,
            color: [
                Color::WHITE,
                Color::RGB(169,169,169),
                Color::RGB(105,105,105),
                Color::BLACK,
            ],
            transparent: Color::RGBA(0, 0, 0, 0),
        };

        renderer
    }

    pub fn store(&mut self, location: usize, value: u8) {
        self.tile_data[location-0x8000] = value;
    }


    fn get_byte_from_location(&self, location:usize) -> u8 {
        self.tile_data[(location-0x8000) % self.tile_data.len()]
    }

    pub fn set_lcdc(&mut self, value: u8) {
        self.lcdc = value;
    }

    pub fn get_screen (&self) -> Vec<ColorPosition> {
        let mut screen: Vec<ColorPosition> = Vec::new();
        let background = self.get_background();
        for y in 0..= HEIGHT {
            for x in 0..= WIDTH {
                let tile: Tile = background[usize::try_from(y/8).unwrap()][usize::try_from(x/8).unwrap()];
                let color = tile.pixels[usize::try_from(y%8).unwrap()][usize::try_from(x%8).unwrap()];
                let clr_position = ColorPosition {
                    y,
                    x,
                    color,
                };
                screen.push(clr_position);
            }
        }

        screen
    }

    fn get_background(&self) -> [[Tile; 32]; 32] {
        let data_area = self.get_background_tile_data_area();
        let map_area = self.get_background_tile_map_area();
        let map_index = if map_area {0x9800} else {0x9c00};
        let data_index:usize = if data_area {0x8000} else {0x8800};
        let mut background: [[Tile; 32]; 32] = [[Tile::new(); 32]; 32];
        for i in 0..0x300 {
            let tile_number: usize = usize::from(self.get_byte_from_location(i + map_index));
            let tile_data = self.get_tile(tile_number * 16 + data_index);
            let x = i % 32;
            let y = i / 32;
            background[y][x] = self.convert_tile(tile_data);
        }

        background
    }

    /// get the next 16 bites that form 1 tile
    fn get_tile(&self, location: usize) -> [u8; 16] {
        let mut tile:[u8; 16] = [0; 16];
        for i in 0..16 {
            tile[i] = self.get_byte_from_location(i + location);
        }

        tile
    }

    /// get background tile map area FALSE is area 0 TRUE is area 1
    fn get_background_tile_map_area(&self) -> bool {
        let num = self.lcdc & 0x8;
        num == 0x8
    }

    /// get background tile data area FALSE is area 0 TRUE is area 1
    fn get_background_tile_data_area(&self) -> bool {
        let num = self.lcdc & 0x10;
        num == 0x10
    }

    fn convert_tile(&self, tile_data: [u8; 16]) -> Tile {
        let mut colors: [[Color; 8]; 8] = [[Color::WHITE; 8]; 8];
        let mut i: usize = 0;
        for byte in tile_data {
            for number in 0..4 {
                let color = self.get_color_from_byte(byte, &number);
                let x = i % 8;
                let y = i / 8;
                colors[y][x] = color;
                i += 1;
            }
        }

        Tile { pixels: colors }
    }

    fn get_color_from_byte(&self, byte: u8, number: &u8) -> Color {
        let get_color = (byte >> number * 2) & 3;
        self.color[usize::from(get_color)]
    }

}