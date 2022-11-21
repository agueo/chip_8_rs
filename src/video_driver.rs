use sdl2::rect::Rect;
use sdl2::{Sdl, render::Canvas, video::Window, pixels::Color};

const SCALE_FACTOR: u32 = 15;
const SCREEN_WIDTH: u32 = 64 * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = 32 * SCALE_FACTOR;

use crate::CHIP_8_HEIGHT;
use crate::CHIP_8_WIDTH;

pub struct VideoDriver {
    canvas: Canvas<Window>
}

impl VideoDriver {
    pub fn new(sdl_context: &Sdl) -> VideoDriver {
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window("chip-8-rs", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0,0,0));
        canvas.clear();
        canvas.present();

        VideoDriver { canvas }
    }

    pub fn draw(&mut self, pixels: &[[u8; CHIP_8_WIDTH]; CHIP_8_HEIGHT]) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                let x = (x as u32) * SCALE_FACTOR;
                let y = (y as u32) * SCALE_FACTOR;
                self.canvas.set_draw_color(get_color(pixel));
                let _ = self.canvas.fill_rect(Rect::new(x as i32, y as i32, SCALE_FACTOR, SCALE_FACTOR));
            }
        }
        self.canvas.present();
    }
}

fn get_color(pixel: u8) -> Color{
    if pixel > 0 {
        return Color::RGB(0, 225, 0);
    }
    Color::RGB(0, 0, 0)
}