extern crate sdl2;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use rand::{thread_rng, Rng};

use sdl2::event::Event;
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;

type WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;

const TILE_SIZE: u32 = 128;
const TILE_PADDING: u32 = 16;

const WINDOW_SIZE: u32 = (TILE_SIZE * 4) + (TILE_PADDING * 5);

lazy_static! {
    static ref color_map: HashMap<u64, Color> = {
        let mut m = HashMap::new();
        m.insert(2,  Color::RGB(0xEE, 0xE4, 0xDA));
        m.insert(4,  Color::RGB(0xED, 0xE0, 0xC8));
        m.insert(8,  Color::RGB(0xF2, 0xB1, 0x79));
        m.insert(16, Color::RGB(0xF5, 0x95, 0x63));
        m
    };
}

struct Grid {
    grid: [[u64; 4]; 4],
}

impl Grid {
    fn empty() -> Grid {
        Grid { grid: [[0; 4]; 4], }
    }
    fn draw(&self, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::RGB(0xCD, 0xC0, 0xB4));
        for row in 0..4 {
            for column in 0..4 {
                if self.grid[row][column] == 0 {
                    canvas.set_draw_color(Color::RGB(0xCD, 0xC0, 0xB4));
                } else {
                    canvas.set_draw_color(
                        *color_map.get(&self.grid[row][column]).unwrap());
                }
                canvas.fill_rect(Rect::new(
                    ((column as i32) * (TILE_SIZE + TILE_PADDING) as i32) +
                        (TILE_PADDING as i32),
                    ((row    as i32) * (TILE_SIZE + TILE_PADDING) as i32) +
                        (TILE_PADDING as i32),
                    TILE_SIZE, TILE_SIZE,
                ));
            }
        }
    }
    fn insert_random_tile(&mut self) {
        let mut rng = thread_rng();
        let row = rng.gen_range(0, 4);
        let column = rng.gen_range(0, 4);
        let choice: u64 = *rng.choose(&[2, 4]).unwrap();
        self.grid[row][column] = choice;
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();
    let sdl_image_context = image::init(image::InitFlag::all()).unwrap();
    let window = video_system
        .window("SDL2 from Rust", WINDOW_SIZE, WINDOW_SIZE)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut grid = Grid::empty();
    grid.insert_random_tile();
    
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::W), .. } |
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    println!("UP");
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } |
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    println!("LEFT");
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } |
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    println!("DOWN");
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } |
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    println!("RIGHT");
                },
                _ => (),
            }
        }
        canvas.set_draw_color(Color::RGB(0xBB, 0xAD, 0xA0));
        canvas.fill_rect(Rect::new(
            0, 0, WINDOW_SIZE, WINDOW_SIZE
        )).expect("fill_rect() failed somehow");
        grid.draw(&mut canvas);
        canvas.present();
    }
}
