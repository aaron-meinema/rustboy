mod cpu;
mod cardridge;
mod memory_map;
mod renderer;

extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;

pub fn main() {
    let vec1:Vec<u8> = vec![0x40, 0x41, 0x42];
    let cardridge = cardridge::Cardridge{
        memory: vec1,
    };
    let mut cpu = cpu::Cpu::new(cardridge);

    cpu.start_cycle();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("rust-sdl2 demo", renderer::WIDTH.try_into().unwrap(), renderer::HEIGHT.try_into().unwrap())
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
        for item in render {
            canvas.set_draw_color(item.color);

            canvas.draw_point(Point::new(item.x, item.y));
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
