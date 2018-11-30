extern crate rand;
extern crate sdl2;

#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap, VecDeque};

use rand::{thread_rng, Rng};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;

use sdl2::ttf;

type WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;

const TILE_SIZE: usize = 128;
const TILE_PADDING: usize = 16;

const WINDOW_SIZE: usize = (TILE_SIZE * 4) + (TILE_PADDING * 5);

lazy_static! {
    static ref color_map: HashMap<u64, Color> = {
        let mut m = HashMap::new();
        m.insert(2, Color::RGB(0xEE, 0xE4, 0xDA));
        m.insert(4, Color::RGB(0xED, 0xE0, 0xC8));
        m.insert(8, Color::RGB(0xF2, 0xB1, 0x79));
        m.insert(16, Color::RGB(0xF5, 0x95, 0x63));
        m.insert(32, Color::RGB(0xF5, 0x7C, 0x5F));
        m.insert(64, Color::RGB(0xF6, 0x5D, 0x3B));
        m.insert(128, Color::RGB(0xEC, 0xCF, 0x6D));
        m.insert(256, Color::RGB(0xED, 0xCC, 0x63));
        m.insert(512, Color::RGB(0xEC, 0xC8, 0x50));
        m.insert(1024, Color::RGB(0xED, 0xC4, 0x3F));
        m.insert(2048, Color::RGB(0xEE, 0xC2, 0x2E));
        m
    };
}

struct v2 {
    row: i32,
    column: i32,
}

impl v2 {
    fn new(row: i32, column: i32) -> v2 {
        v2 {
            row: row,
            column: column,
        }
    }
}

#[derive(Copy, Clone)]
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
    fn at(&self, place: &v2) -> u64 {
        self.grid[place.row as usize][place.column as usize]
    }
    fn draw(&self, canvas: &mut WindowCanvas, font: &ttf::Font) {
        let dark_text_color: Color = Color::RGB(0x78, 0x6f, 0x66);
        let light_text_color: Color = Color::RGB(0xFD, 0xF3, 0xF1);

        fn grid_to_real(pos: (usize, usize)) -> (usize, usize) {
            (
                pos.0 * (TILE_SIZE + TILE_PADDING) + TILE_PADDING,
                pos.1 * (TILE_SIZE + TILE_PADDING) + TILE_PADDING,
            )
        }

        canvas.set_draw_color(Color::RGB(0xCD, 0xC0, 0xB4));

        for row in 0..GRID_SIZE {
            for column in 0..GRID_SIZE {
                // Draw squares
                if self.grid[row][column] == 0 {
                    canvas.set_draw_color(Color::RGB(0xCD, 0xC0, 0xB4));
                } else {
                    canvas.set_draw_color(*color_map.get(&self.grid[row][column]).unwrap());
                }
                let pos = grid_to_real((column, row));
                canvas
                    .fill_rect(Rect::new(
                        pos.0 as i32,
                        pos.1 as i32,
                        TILE_SIZE as u32,
                        TILE_SIZE as u32,
                    )).unwrap();
                // Draw numbers
                if self.grid[row][column] > 0 {
                    let texture_creator = canvas.texture_creator();
                    let texture: sdl2::render::Texture = {
                        let text = format!("{}", self.grid[row][column]);
                        let partial = font.render(text.as_str());
                        let surface = partial
                            .blended(if self.grid[row][column] <= 4 {
                                dark_text_color
                            } else {
                                light_text_color
                            }).unwrap();
                        texture_creator
                            .create_texture_from_surface(surface)
                            .unwrap()
                    };
                    let texture_size = {
                        let query = texture.query();
                        (query.width, query.height)
                    };
                    canvas.copy(
                        &texture,
                        None,
                        Some(Rect::new(
                            pos.0 as i32 + (TILE_SIZE as i32 / 2) - (texture_size.0 as i32 / 2),
                            pos.1 as i32 + (TILE_SIZE as i32 / 2) - (texture_size.1 as i32 / 2),
                            texture_size.0,
                            texture_size.1,
                        )),
                    );
                }
            }
        }
    }
    fn shift_block(&mut self, place: v2, dir: v2) -> bool {
        let peek = v2::new(place.row + dir.row, place.column + dir.column);
        if self.at(&place) != 0 {
            if self.at(&peek) == 0 {
                self.grid[peek.row as usize][peek.column as usize] = self.at(&place);
                self.grid[place.row as usize][place.column as usize] = 0;
                true
            } else if self.at(&peek) == self.at(&place) {
                self.grid[peek.row as usize][peek.column as usize] = self.at(&place) * 2;
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
                    if self.shift_block(v2::new(row as i32, column), v2::new(0, dir)) {
                        changes_made = true;
                    }
                }
            }
            if !changes_made {
                break;
            }
        }
    }
    fn shift_rows(&mut self, start: usize, end: usize, dir: i32) {
        if start as i32 + dir < 0
            || start as i32 + dir >= GRID_SIZE as i32
            || end as i32 + dir < 0
            || end as i32 + dir >= GRID_SIZE as i32
        {
            panic!("Invalid start or end for direction");
        }
        loop {
            let mut changes_made = false;
            for column in 0..GRID_SIZE {
                for row in BetterRange::new(start as i32, end as i32) {
                    if self.shift_block(v2::new(row, column as i32), v2::new(dir, 0)) {
                        changes_made = true;
                    }
                }
            }
            if !changes_made {
                break;
            }
        }
    }
    fn shift(&mut self, dir: Direction) {
        match dir {
            Direction::West => self.shift_columns(1, GRID_SIZE - 1, -1),
            Direction::East => self.shift_columns(GRID_SIZE - 2, 0, 1),
            Direction::North => self.shift_rows(1, GRID_SIZE - 1, -1),
            Direction::South => self.shift_rows(GRID_SIZE - 2, 0, 1),
        }
    }
    fn is_full(&self) -> bool {
        for row in 0..GRID_SIZE {
            for column in 0..GRID_SIZE {
                if self.at(&v2::new(row as i32, column as i32)) == 0 {
                    return false;
                }
            }
        }
        return true;
    }
    fn insert_random_tile(&mut self) {
        if self.is_full() {
            return;
        }
        let mut rng = thread_rng();
        let mut row;
        let mut column;
        loop {
            row = rng.gen_range(0, GRID_SIZE);
            column = rng.gen_range(0, GRID_SIZE);
            if self.at(&v2::new(row as i32, column as i32)) == 0 {
                break;
            }
        }
        let choice: u64 = *rng.choose(&[2, 4]).unwrap();
        self.grid[row][column] = choice;
    }
}

fn make_moves(grid: &mut Grid, move_queue: &mut VecDeque<Direction>) {
    while !move_queue.is_empty() {
        let direction = move_queue.pop_front().unwrap();
        grid.shift(direction);
        grid.insert_random_tile();
    }
}

trait GameState {
    fn event(&mut self, event: Event);
    fn update(&mut self);
    fn render(&mut self, canvas: &mut WindowCanvas, font: &ttf::Font);
}

struct PlayingState {
    move_queue: VecDeque<Direction>,
    grid: Grid,
}

trait Stack<T> {
    fn top(&self) -> Option<&T>;
}

impl<T> Stack<T> for Vec<T> {
    fn top(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self[self.len() - 1])
        }
    }
}

fn main() {
    let mut state_stack: Vec<Box<dyn GameState>> = Vec::new();
    
    let sdl_context = sdl2::init().unwrap();
    let ttf_context: ttf::Sdl2TtfContext = ttf::init().unwrap();

    let font = {
        let font_bytes = include_bytes!("../resources/ClearSans-Medium.ttf");
        ttf_context
            .load_font_from_rwops(sdl2::rwops::RWops::from_bytes(font_bytes).unwrap(), 72)
            .unwrap()
    };

    let mut canvas = {
        let video_system = sdl_context.video().unwrap();
        let window = video_system
            .window("SDL2 from Rust", WINDOW_SIZE as u32, WINDOW_SIZE as u32)
            .position_centered()
            .build()
            .unwrap();
        window.into_canvas().build().unwrap()
    };

    let mut move_queue: VecDeque<Direction> = VecDeque::new();
    let mut grid = Grid::empty();
    grid.insert_random_tile();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            /*
            match state_stack.top() {
                GameStat:e:Playing => 
            }*/
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
                    move_queue.push_back(Direction::North);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    move_queue.push_back(Direction::West);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    move_queue.push_back(Direction::South);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    move_queue.push_back(Direction::East);
                }
                _ => (),
            }
        }
        make_moves(&mut grid, &mut move_queue);
        canvas.set_draw_color(Color::RGB(0xBB, 0xAD, 0xA0));
        canvas
            .fill_rect(Rect::new(0, 0, WINDOW_SIZE as u32, WINDOW_SIZE as u32))
            .unwrap();
        grid.draw(&mut canvas, &font);
        canvas.present();
    }
}
