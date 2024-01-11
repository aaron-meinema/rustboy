mod cpu;
mod cardridge;
mod memory_map;
mod renderer;
mod settings;

extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;

pub fn main() {
    let vec1:Vec<u8> = vec![0x40, 0x41, 0x42];
    let settings = settings::Settings::get_settings();

    let height: u32 = renderer::HEIGHT.try_into().expect("could not convert height i32 to u32");
    let width: u32 = renderer::WIDTH.try_into().expect("could not convert height i32 to u32");
    let cardridge = cardridge::Cardridge{
        memory: vec1,
    };
    let mut cpu = cpu::Cpu::new(cardridge);

    cpu.start_cycle();//70224 clocks then render a new frame
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut window = video_subsystem.window("rust-sdl2 demo",width * settings.render_scale, height * settings.render_scale)
        .position_centered()
        .build()
        .unwrap();
    
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        let render = cpu.memory_map.renderer.get_screen();
        let scale = i32::try_from(settings.render_scale).expect("couldn't convert render scale to i32");
        for item in render {
            let x_location: i32 = item.x * i32::try_from(settings.render_scale).expect("could not conver x location from i32 to u32");
            let y_location: i32 = item.y * i32::try_from(settings.render_scale).expect("could not conver y location from i32 to u32");
            canvas.set_draw_color(item.color);
            for x in 0..scale {
                for y in 0..scale {
                    canvas.draw_point(Point::new(x_location + x, y_location + y));
                }
            }
        }


        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}