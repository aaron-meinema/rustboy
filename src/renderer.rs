use sdl2::pixels::Color;

pub const WIDTH:  i32 = 160;
pub const HEIGHT: i32 = 144;

pub struct Renderer {
    tile_data: [u8; 0x97ff - 0x8000],

    color: [Color; 4],
}

/// priority is for when more than 10 sprites are on a line
struct Pixel {
    priority: u8,
    color: Color,
}

pub struct ColorPosition {
    pub y: i32,
    pub x: i32,

    pub color: Color
}


struct Sprite {
    x_location: u8,
    y_location: u8,
}

impl Renderer {
    pub fn new() -> Self {
        let tile:[u8; 0x97ff - 0x8000]  = [0; 0x97ff - 0x8000];
        let renderer = Renderer {
            tile_data: tile,
            color: [
                Color::RGB(0, 0, 0),
                Color::RGB(255, 0, 0),
                Color::RGB(0, 255, 0),
                Color::RGB(0, 0, 255),
            ]
        };

        renderer
    }

    pub fn store_data(&mut self, location: usize, value: u8) {
        self.tile_data[location-0x8000] = value;
    }

    pub fn get_screen (&self) -> Vec<ColorPosition> {
        let mut screen: Vec<ColorPosition> = Vec::new();
        for y in 0..= HEIGHT * 2 {
            let index = usize::try_from(y%4).unwrap();
            let color = self.color[index];
            for x in 0..= WIDTH * 2 {
                let clr_position = ColorPosition {
                    y,
                    x,
                    color,
                };
                screen.push(clr_position);
            }
        }
        for x in 0..WIDTH*2 {
            let index = usize::try_from(x%4).unwrap();
            let color = self.color[index];
            for y in 0..HEIGHT*2 {
            }
        }

        screen
    }

    fn get_background(&self) {
        todo!("implement");
    }



    fn get_tile(&self, number: usize) -> [u8; 16]{
        let full_number = number * 16;
        let mut tile:[u8; 16] = [0; 16];
        for n in 0..16 {
            tile[n] = self.tile_data[n + full_number];
        }

        tile
    }
}