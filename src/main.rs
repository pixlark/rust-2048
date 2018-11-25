extern crate rand;
extern crate sdl2;

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
        m.insert(2, Color::RGB(0xEE, 0xE4, 0xDA));
        m.insert(4, Color::RGB(0xED, 0xE0, 0xC8));
        m.insert(8, Color::RGB(0xF2, 0xB1, 0x79));
        m.insert(16, Color::RGB(0xF5, 0x95, 0x63));
        m
    };
}

struct v2 {
    row: i32,
    column: i32,
}

impl v2 {
    fn new(row: i32, column: i32) -> v2 {
        v2 { row: row, column: column }
    }
}

enum Direction {
    North,
    West,
    South,
    East,
}

struct BetterRange {
    start: i32,
    end: i32,
    step: i32,
}

impl BetterRange {
    fn new(start: i32, end: i32) -> BetterRange {
        BetterRange {
            start: start,
            end: end,
            step: (end - start).abs() / (end - start),
        }
    }
}

impl Iterator for BetterRange {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.start == self.end + self.step {
            None
        } else {
            let ret = self.start;
            self.start += self.step;
            Some(ret)
        }
    }
}

const GRID_SIZE: usize = 4;

struct Grid {
    grid: [[u64; GRID_SIZE]; GRID_SIZE],
}

impl Grid {
    fn empty() -> Grid {
        Grid {
            grid: [[0; GRID_SIZE]; GRID_SIZE],
        }
    }
    fn draw(&self, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::RGB(0xCD, 0xC0, 0xB4));
        for row in 0..GRID_SIZE {
            for column in 0..GRID_SIZE {
                if self.grid[row][column] == 0 {
                    canvas.set_draw_color(Color::RGB(0xCD, 0xC0, 0xB4));
                } else {
                    canvas.set_draw_color(*color_map.get(&self.grid[row][column]).unwrap());
                }
                canvas
                    .fill_rect(Rect::new(
                        ((column as i32) * (TILE_SIZE + TILE_PADDING) as i32)
                            + (TILE_PADDING as i32),
                        ((row as i32) * (TILE_SIZE + TILE_PADDING) as i32) + (TILE_PADDING as i32),
                        TILE_SIZE,
                        TILE_SIZE,
                    )).unwrap();
            }
        }
    }
    fn shift_block(&mut self, place: v2, dir: v2) -> bool {
        if self.grid[place.row as usize][place.column as usize] != 0 {
            if self.grid[(place.row + dir.row) as usize][(place.column + dir.column) as usize] == 0
            {
                self.grid[(place.row + dir.row) as usize][(place.column + dir.column) as usize] =
                    self.grid[place.row as usize][place.column as usize];
                self.grid[place.row as usize][place.column as usize] = 0;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    fn shift_columns(&mut self, start: usize, end: usize, dir: i32) {
        //println!("shifting columns {}", dir);
        if start as i32 + dir < 0
            || start as i32 + dir >= GRID_SIZE as i32
            || end as i32 + dir < 0
            || end as i32 + dir >= GRID_SIZE as i32
        {
            panic!("Invalid start or end for direction");
        }
        loop {
            let mut changes_made = false;
            for column in BetterRange::new(start as i32, end as i32) {
                for row in 0..GRID_SIZE {
                    if self.shift_block(v2::new(row as i32, column), v2::new(row as i32, column + dir)) {
                        changes_made = true;
                    }
                }
            }
            if !changes_made {
                break;
            }
        }
    }
    fn shift_rows(&mut self, start: usize, end: usize, dir: i32) {}
    fn shift(&mut self, dir: Direction) {
        match dir {
            Direction::West => self.shift_columns(1, GRID_SIZE - 1, -1),
            Direction::East => self.shift_columns(GRID_SIZE - 2, 0, 1),
            Direction::North => self.shift_rows(1, GRID_SIZE - 1, -1),
            Direction::South => self.shift_rows(GRID_SIZE - 2, 0, 1),
        }
    }
    fn insert_random_tile(&mut self) {
        let mut rng = thread_rng();
        let row = rng.gen_range(0, GRID_SIZE);
        let column = rng.gen_range(0, GRID_SIZE);
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
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    grid.shift(Direction::North);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    grid.shift(Direction::West);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    grid.shift(Direction::South);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    grid.shift(Direction::East);
                }
                _ => (),
            }
        }
        canvas.set_draw_color(Color::RGB(0xBB, 0xAD, 0xA0));
        canvas
            .fill_rect(Rect::new(0, 0, WINDOW_SIZE, WINDOW_SIZE))
            .unwrap();
        grid.draw(&mut canvas);
        canvas.present();
    }
}
