mod cpu;
mod cardridge;
mod memory_map;
mod renderer;
mod settings;

extern crate sdl2;
use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{WindowContext, Window};
use settings::Settings;

use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

pub struct Sdl2Helper {
    settings: Settings,
    width: u32,
    debug_message: Vec<String>,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    texture_creator: TextureCreator<WindowContext>
}

impl Sdl2Helper {
    pub fn new() -> Self {
        let sdl_con = sdl2::init().unwrap();
        let video = sdl_con.video().unwrap();
        let settings = settings::Settings::get_settings();
        let height: u32 = renderer::HEIGHT.try_into().expect("could not convert height i32 to u32");
        let width = renderer::WIDTH.try_into().expect("could not convert height i32 to u32");
        let win = video.window("rustboy",width * settings.render_scale, height * settings.render_scale)
            .position_centered()
            .build()
            .unwrap();

        let canvas = win.into_canvas().build().unwrap();
        let event_pump = sdl_con.event_pump().unwrap();
        let texture_creator = canvas.texture_creator();
        let sdl = Sdl2Helper {
            settings,
            width,
            debug_message: Vec::new(),
            canvas,
            event_pump,
            texture_creator
        };
        return sdl;
    }

    fn get_centered_rect(&self, location: u32, horizon: u32) -> Rect {
        let (w, h) =  (64, 24);
        
        let cx = horizon;
        let cy = location * 10;
        rect!(cx, cy, w, h)
    }

    fn print_debug_messages(&mut self) {
        let mut index = 0;
        for ele in &self.debug_message {
            let target = self.get_centered_rect(index, self.width*self.settings.render_scale - 15 * self.settings.render_scale);
            let surface = sdl2::ttf::init().map_err(|e| e.to_string()).expect("font error").load_font("Roboto-Regular.ttf", 200).expect("msg")
            .render(ele)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).unwrap();
        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

            self.canvas.copy(&texture, None, Some(target)).unwrap();
            
            index += 2;
        }
        self.debug_message = Vec::new();
    }

    fn add_debug_message(&mut self, value: String) {
        self.debug_message.push(value);
    }
    

}
fn get_rom() -> Vec<u8> {
    let args: Vec<String> = env::args().collect();
    let binding = "default".to_string();
    let arg = args.get(1).unwrap_or(&binding);
    let mut buffer: Vec<u8> = Vec::new();
    if arg.eq("default") {
        return vec![0x40, 0x41, 0x42];
    }
    let file = File::open(arg);
    match file {
        Ok(mut val) => {
            let _ = val.read_to_end(&mut buffer);
            return buffer;
        },
        Err(_) => return vec![0x40, 0x41, 0x42],
    }
}
pub fn main() {
    let vec1:Vec<u8> = get_rom();
    let mut sdl_help = Sdl2Helper::new();
    let cardridge = cardridge::Cardridge{
        memory: vec1,
    };
    let mut cpu = cpu::Cpu::new(cardridge);

    let mut buttons: u8 = 0x0f;
    let mut d_pad: u8 = 0x0f;


    'running: loop {
        sdl_help.add_debug_message(format!("{:#04x}", &cpu.memory_map.get_8bit_full_address(0xff00)).as_str().to_string());
        sdl_help.add_debug_message("hello world".to_owned());
        cpu.start_cycle();

        sdl_help.canvas.clear();
        sdl_help.canvas.set_draw_color(Color::RGB(0, 0, 0));
        let render = cpu.memory_map.renderer.get_screen();
        let scale = i32::try_from(sdl_help.settings.render_scale).expect("couldn't convert render scale to i32");
        for item in render {
            let x_location: i32 = item.x * i32::try_from(sdl_help.settings.render_scale).expect("could not conver x location from i32 to u32");
            let y_location: i32 = item.y * i32::try_from(sdl_help.settings.render_scale).expect("could not conver y location from i32 to u32");
            sdl_help.canvas.set_draw_color(item.color);
            for x in 0..scale {
                for y in 0..scale {
                    let _ = sdl_help.canvas.draw_point(Point::new(x_location + x, y_location + y));
                }
            }
        }
        
        sdl_help.print_debug_messages();
        for event in sdl_help.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Z), .. } => {       //A
                    buttons &= 0xfe;
                },
                Event::KeyUp { keycode: Some(Keycode::Z), .. } => {         //A
                    buttons |= 1;
                },
                Event::KeyDown { keycode: Some(Keycode::X), .. } => {       //B
                    buttons &= 0xfd;
                },
                Event::KeyUp { keycode: Some(Keycode::X), .. } => {         //B
                    buttons |= 2;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {  //Select
                    buttons &= 0xfb;
                },
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => {    //Select
                    buttons |= 4;
                }, 
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => { //Start
                    buttons &= 0xf7;
                },
                Event::KeyUp { keycode: Some(Keycode::Return), .. } => {   //Start
                    buttons |= 8;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {  //Right
                    d_pad &= 0xfe;
                },
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => {    //Right
                    d_pad |= 1;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {   //Left
                    d_pad &= 0xfd;
                },
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => {     //Left
                    d_pad |= 2;
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {      //UP
                    d_pad &= 0xfb;
                },
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => {        //UP
                    d_pad |= 4;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {    //Down
                    d_pad &= 0xf7;
                },
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => {      //DOWN
                    d_pad |= 8;
                },
                
                _ => {}
            }
        }
        cpu.memory_map.store_buttons(buttons);
        cpu.memory_map.store_d_pad(d_pad);
        // The rest of the game loop goes here...
        sdl_help.canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}



