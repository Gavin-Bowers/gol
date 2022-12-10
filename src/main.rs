use std::collections::HashMap;
use std::collections::HashSet;
use std::vec;

use ggez::event::EventHandler;
use rand::Rng;

use ggez::conf;
use ggez::event;
use ggez::timer;
use ggez::graphics::{self, Color};
//use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult};

const TILES_SPANNING: u32 = 500; //The number of tiles spanning the board
const TILE_SIZE: u32 = 2; //Width of one tile in Pixels
const SCREEN_SIZE: f32 = 
    (TILES_SPANNING * TILE_SIZE) as f32;

//type Cell = (i32, i32);

#[derive(PartialEq, Eq, Hash)]
struct Cell {
    x: i32,
    y: i32
}

impl Cell {
    pub fn new(x: &i32, y: &i32) -> Self {
        Cell{x: *x,y: *y}
    }
    /*
    pub fn consume(x: i32, y: i32) -> Self {
        Cell{x,y}
    }
    */
    fn draw(&self, canvas: &mut graphics::Canvas) {
        canvas.draw(
        &graphics::Quad,
        graphics::DrawParam::new()
            .dest_rect(self.into())
            .color([1.,1.,1.,1.]),
        );
    }
}

impl From<&Cell> for graphics::Rect { //Helper function to turn cell into a rect so it can draw itself
    fn from(cell_to_draw: &Cell) -> Self {
        graphics::Rect::new_i32(
            TILE_SIZE as i32 * cell_to_draw.x,
            TILE_SIZE as i32 * cell_to_draw.y,
            TILE_SIZE as i32,
            TILE_SIZE as i32,
        )
    }
}

//################################################
//Supporting functions which assist generation
//################################################

fn neighbours(cell: &Cell) -> Vec<Cell> {
    let x: i32 = cell.x;
    let y: i32 = cell.y;
    vec![
    Cell::new(&(x-1),&(y-1)), Cell::new(&x,&(y-1)), Cell::new(&(x+1),&(y-1)),
    Cell::new(&(x-1),&y),                           Cell::new(&(x+1),&y),
    Cell::new(&(x-1),&(y+1)), Cell::new(&x,&(y+1)), Cell::new(&(x+1),&(y+1)),
    ]
}

fn neighbour_counts(col: &HashSet<Cell>) -> HashMap<Cell, i32> {
    let mut ncnts = HashMap::new();
    for cell in col.iter().flat_map(neighbours) {
        *ncnts.entry(cell).or_insert(0) += 1;
    }
    ncnts
}

fn generation(col: &HashSet<Cell>) -> HashSet<Cell> {
    neighbour_counts(&col)
        .into_iter()
        .filter_map(|(cell, cnt)|
            match (cnt, col.contains(&cell)) {
                (2, true) |
                (3, ..) => Some(cell),
                _ => None
        })
        .collect()
}

fn random_cells() -> HashSet<Cell> {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Cell> = Vec::new();
    for row in 0..TILES_SPANNING 
    {
        for column in 0..TILES_SPANNING
        {
            if rng.gen_range(0..2) == 1 {
                cells.push(Cell::new(&(row as i32),&(column as i32)));
            }
        }
    }
    let cell_set: HashSet<Cell> = cells.into_iter().collect();
    cell_set
}

struct MainState { //Used to contain all the information of the game while running
    screen: graphics::ScreenImage,
    col: HashSet<Cell>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let screen = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb,1.,1.,1);
        //let col = HashSet::<Cell>::new();
        let col = random_cells();
        
        let state = MainState {
            screen,
            col,
        };
        Ok(state)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 30; //Using framerates higher than 30 cause terrible lag for some reason

        while ctx.time.check_update_time(DESIRED_FPS) {
            //let seconds = 1.0 / (DESIRED_FPS as f32);
            let new_col = generation(&self.col);
            self.col = new_col;

        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_screen_image(ctx, &mut self.screen, Color::BLACK);

        for cell in self.col.iter() {
            cell.draw(&mut canvas);
        }

        canvas.finish(ctx)?;
        ctx.gfx.present(&self.screen.image(ctx))?;

        timer::yield_now();
        Ok(())
    }
}

fn main() -> GameResult {
    /*
    let glider = vec![
        Cell::consume(1,0),
                                                  Cell::consume(2,1),
        Cell::consume(0,2),  Cell::consume(1,2),  Cell::consume(2,2)];
    */
    let context_builder = ContextBuilder::new("game_of_life", "ggez")
    .window_setup(conf::WindowSetup::default().title("Game of Life"))
    .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE, SCREEN_SIZE));

    let (mut ctx, events_loop) = context_builder.build()?;

    let game_state = MainState::new(&mut ctx)?;

    event::run(ctx, events_loop, game_state)
}

//life(glider, 20, 8, 8);

//life(random_cells(15,15), 20, 15, 15);
