use sdl2::pixels::Color;
use std::cmp::Ordering;
use std::fmt::Debug;

pub const WIDTH: i32 = 160;
pub const HEIGHT: i32 = 144;
pub const DEBUG: i32 = 30;

pub struct Renderer {
    tile_data: [u8; 0x9fff - 0x8000],
    oam_data: [u8; 0xfea0 - 0xfe00],
    lcdc: u8,
    color: [Color; 4],
    transparent: Color,
}
#[derive(Debug)]
pub struct ColorPosition {
    pub y: i32,
    pub x: i32,

    pub color: Color,
}

impl ColorPosition {
    fn new(x: i32, y: i32, color: Color) -> Self {
        ColorPosition { y, x, color }
    }
}

struct Sprite {
    x: i32,
    y: i32,
    tile: Tile,
    flags: u8,
}

impl Sprite {
    pub fn new(x: i32, y: i32, tile: Tile, flags: u8) -> Self {
        Sprite { x, y, tile, flags }
    }

    fn compare(&self, x: i32, y: i32, compare_sprite: &Sprite) -> Ordering {
        if self.get_pixel(x, y).eq(&Color::WHITE) {
            Ordering::Less
        } 
        else if self.x > compare_sprite.x {
            Ordering::Less
        }
        else {
            Ordering::Greater
        }
    }

    fn compare_bool(&self, x: i32, y: i32, eight_by_sixteen: bool) -> bool {
        let y_size = if eight_by_sixteen { 16 } else { 8 };
        if self.x <= x && self.x + 8 > x && self.y <= y && self.y + y_size > y {
            true
        } else {
            false
        }
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        let y_location = usize::try_from(y - self.y).unwrap();
        let x_location = usize::try_from(x - self.x).unwrap();
        self.tile.pixels[y_location][x_location]
    }

    fn over_background_foreground(&self) -> bool {
        (self.flags & 0x80) == 0x80
    }

    fn yflip(&self) -> bool {
        (self.flags & 0x40) == 0x40
    }

    fn xflip(&self) -> bool {
        (self.flags & 0x20) == 0x20
    }

    fn palette_number(&self) -> bool {
        (self.flags & 0x10) == 0x10
    }
}

struct Tile {
    pixels: [[Color; 8]; 8],
}

impl Clone for Tile {
    fn clone(&self) -> Self {
        Self {
            pixels: self.pixels.clone(),
        }
    }
}

impl Copy for Tile {}

impl Tile {
    pub fn new() -> Self {
        let color = Color::WHITE;
        Tile {
            pixels: [[color; 8]; 8],
        }
    }
}

impl Renderer {
    pub fn new() -> Self {
        let tile: [u8; 0x9fff - 0x8000] = [0; 0x9fff - 0x8000];
        let oam: [u8; 0xfea0 - 0xfe00] = [0; 0xfea0 - 0xfe00];
        let renderer = Renderer {
            tile_data: tile,
            oam_data: oam,
            lcdc: 0,
            color: [
                Color::WHITE,
                Color::RGB(169, 169, 169),
                Color::RGB(105, 105, 105),
                Color::BLACK,
            ],
            transparent: Color::RGBA(0, 0, 0, 0),
        };

        renderer.get_all_sprites();
        renderer
    }

    pub fn store(&mut self, location: usize, value: u8) {
        self.tile_data[location - 0x8000] = value;
    }

    fn get_byte_from_location(&self, location: usize) -> u8 {
        self.tile_data[(location - 0x8000) % self.tile_data.len()]
    }

    pub fn set_lcdc(&mut self, value: u8) {
        self.lcdc = value;
    }

    pub fn get_screen(&self) -> Vec<ColorPosition> {
        let mut screen: Vec<ColorPosition> = Vec::new();
        let background = self.get_background();
        for y in 0..=HEIGHT {
            for x in 0..=WIDTH {
                let tile: Tile =
                    background[usize::try_from(y / 8).unwrap()][usize::try_from(x / 8).unwrap()];
                let color =
                    tile.pixels[usize::try_from(y % 8).unwrap()][usize::try_from(x % 8).unwrap()];
                let clr_position = ColorPosition { y, x, color };

                screen.push(clr_position);
            }
        }
        let oke = self.get_sprites_from_screen();
        screen
    }

    fn get_sprites_from_screen(&self) -> Vec<ColorPosition> {
        let eight_by_sixteen = self.get_object_size();
        let mut screen: Vec<ColorPosition> = Vec::new();
        let sprites = self.get_all_sprites();
        for y in 0..=HEIGHT {
            let mut pixel_count = 0;
            for x in 0..=WIDTH {
                let found_sprites: Vec<&Sprite> = sprites
                    .iter()
                    .filter(|sprite| sprite.compare_bool(x, y, eight_by_sixteen))
                    .collect();
                if found_sprites.len() == 0 {
                    continue;
                }
                if pixel_count >= 10 {
                    break;
                }
                pixel_count += 1;
                if x < 0 || y < 0 {
                    continue;
                }
                let sprite = *found_sprites
                                    .iter()
                                    .max_by(|s1, s2| s1.compare(x, y, *s2))
                                    .unwrap();
                screen.push(ColorPosition::new(x, y, sprite.get_pixel(x, y)));
            }
        }
        screen
    }

    fn get_all_sprites(&self) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = Vec::new();
        for i in 0..40 {
            sprites.push(self.get_sprite(i));
        }
        sprites.sort_by(|a, b| a.x.cmp(&b.x));
        sprites
    }

    fn get_background(&self) -> [[Tile; 32]; 32] {
        let data_area = self.get_background_tile_data_area();
        let map_area = self.get_background_tile_map_area();
        let map_index = if map_area { 0x9800 } else { 0x9c00 };
        let data_index: usize = if data_area { 0x8000 } else { 0x8800 };
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

    fn get_sprite(&self, number: usize) -> Sprite {
        let oam_offset = number * 4;
        let tile = self.get_tile(<u8 as Into<usize>>::into(self.oam_data[oam_offset + 2]) + 0x8000);
        let x = i32::from(self.oam_data[oam_offset]) - 8;
        let y = i32::from(self.oam_data[oam_offset + 1]) - 16;
        let tile = self.convert_tile(tile);
        let flags = self.oam_data[oam_offset + 3];

        Sprite::new(x, y, tile, flags)
    }

    /// get the next 16 bites that form 1 tile
    fn get_tile(&self, location: usize) -> [u8; 16] {
        let mut tile: [u8; 16] = [0; 16];
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

    /// returns object size flag in a bool true 8x16 false 8x8
    fn get_object_size(&self) -> bool {
        let num = self.lcdc & 0x4;
        num == 0x4
    }

    /// are objects enabled
    fn get_object_enable(&self) -> bool {
        let num = self.lcdc & 0x2;
        num == 0x2
    }

    /// get window priority
    fn get_window_background_priority(&self) -> bool {
        let num = self.lcdc & 0x1;
        num == 0x1
    }

    /// returns the height of the object based on the lcdc flag
    fn get_object_height(&self) -> i32 {
        if self.get_object_size() {
            16
        } else {
            8
        }
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
