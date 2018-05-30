extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

extern crate chor;

use chor::Chip8;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use std::fs::File;
use std::io;
use std::time::Instant;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
}

impl App {
    fn render(&mut self, args: &RenderArgs, screen: &[bool]) {
        use graphics::*;
        const BG: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 8.0);

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BG, gl);

            let mut x = 0_f64;
            let mut y = 0_f64;
            for pixel in screen.iter() {
                if *pixel {
                    let transform = c.transform.trans(x, y);
                    rectangle(WHITE, square, transform, gl);
                }
                x += 8_f64;
                if x == 64_f64 * 8_f64 {
                    x = 0_f64;
                    y += 8_f64;
                }
            }
        });
    }
}

fn main() -> io::Result<()> {
    let mut chip_8 = Chip8::new();
    let mut file = File::open(std::env::args().skip(1).next().unwrap()).unwrap();
    chip_8.load(&mut file)?;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
        "Chip8",
        [64 * 8, 32 * 8]
    )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
    };

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r, &chip_8.gfx);
            chip_8.render();
        }

        if let Some(_) = e.update_args() {
            chip_8.emulate_cycle();
            chip_8.decrease_timer();
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::D1 => chip_8.press_key(0),
                Key::D2 => chip_8.press_key(1),
                Key::D3 => chip_8.press_key(2),
                Key::D4 => chip_8.press_key(3),
                Key::Q => chip_8.press_key(4),
                Key::W => chip_8.press_key(5),
                Key::E => chip_8.press_key(6),
                Key::R => chip_8.press_key(7),
                Key::A => chip_8.press_key(8),
                Key::S => chip_8.press_key(9),
                Key::D => chip_8.press_key(10),
                Key::F => chip_8.press_key(11),
                Key::Z => chip_8.press_key(12),
                Key::X => chip_8.press_key(13),
                Key::C => chip_8.press_key(14),
                Key::V => chip_8.press_key(15),
                _ => {},
            }
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::D1 => chip_8.release_key(0),
                Key::D2 => chip_8.release_key(1),
                Key::D3 => chip_8.release_key(2),
                Key::D4 => chip_8.release_key(3),
                Key::Q => chip_8.release_key(4),
                Key::W => chip_8.release_key(5),
                Key::E => chip_8.release_key(6),
                Key::R => chip_8.release_key(7),
                Key::A => chip_8.release_key(8),
                Key::S => chip_8.release_key(9),
                Key::D => chip_8.release_key(10),
                Key::F => chip_8.release_key(11),
                Key::Z => chip_8.release_key(12),
                Key::X => chip_8.release_key(13),
                Key::C => chip_8.release_key(14),
                Key::V => chip_8.release_key(15),
                _ => {},
            }
        }
    }
    Ok(())
}
