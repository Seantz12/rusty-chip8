extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Rect;

use super::cpu::DisplayData;

const PIXEL_SIZE: u32 = 24;
const SCREEN_WIDTH: u32 = PIXEL_SIZE * super::WIDTH as u32;
const SCREEN_HEIGHT: u32 = PIXEL_SIZE * super::HEIGHT as u32;
const BLACK: Color = Color::RGB(0,0,0);
const WHITE: Color = Color::RGB(255,255,255);

/* 
 * A lot of this display code heavily refrences Starr Horne's display driver for
 * their CHIP 8 emulator, a link to their repo can be found in the readme.
 */ 

pub struct Display {
    canvas: Canvas<Window>
}

impl Display {
    pub fn new() -> Display {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
    
        let window = video_subsystem.window("CHIP8", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();
    
        let mut canvas = window.into_canvas().build().unwrap();
    
        canvas.set_draw_color(BLACK);
        canvas.clear();
        canvas.present();
        Display {
            canvas: canvas
        }
    }

    pub fn draw(&mut self, display_data: DisplayData) {
        let display = display_data.display;
        for (y, row) in display.iter().enumerate() {
            for (x, color) in row.iter().enumerate() {
                let pos_x: u32 = x as u32 * PIXEL_SIZE;
                let pos_y: u32 = y as u32 * PIXEL_SIZE;
                // println!("Color of pixel: {}", *color); // DEBUG
                if *color != 0 { // draw a pixel
                    // println!("hey im supposed to be uh drawing?"); // DEBUG
                    self.canvas.set_draw_color(WHITE);
                } else {
                    // println!("hey im supposed to be uh drawing but black?"); // DEBUG
                    self.canvas.set_draw_color(BLACK);
                }
                let _ = self.canvas.fill_rect(Rect::new(pos_x as i32, pos_y as i32, PIXEL_SIZE, PIXEL_SIZE));
                // print!("{}", color);
            }
            // println!();
        }
        self.canvas.present();
    }
}