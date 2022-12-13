use std::time::Instant;
use std::vec;

//Concurrency
use dashmap::DashMap;
use dashmap::DashSet;
use rayon::prelude::*;

//RNG
use rand::Rng;

//Graphics, input, and timing
use ggez::conf;
use ggez::event;
use ggez::timer;
use ggez::graphics::{self, Color};
//use ggez::input::keyboard::KeyCode;
use ggez::event::EventHandler;
use ggez::{Context, ContextBuilder, GameResult};

const DESIRED_FPS: u32 = 30;
const TILES_SPANNING: u32 = 300; //The number of tiles spanning the board
const AUTO_SCALE_TILES: bool = true;
const TILE_SIZE: u32 = if AUTO_SCALE_TILES {1000 / TILES_SPANNING} else {10} ; //Width of one tile in Pixels
const SCREEN_SIZE: f32 = (TILES_SPANNING * TILE_SIZE) as f32;

//Performance tools:
const PRINT_FPS: bool = true;
const PRINT_DRAW_TIME: bool = false;

type Cell = (i32, i32);

type Colony = DashSet<Cell>;

fn cell_to_rect(cell: &Cell) -> graphics::Rect {
    graphics::Rect::new_i32(
        TILE_SIZE as i32 * cell.0,
        TILE_SIZE as i32 * cell.1,
        TILE_SIZE as i32,
        TILE_SIZE as i32,
    )
}

fn draw_cell(cell: &Cell, canvas: &mut graphics::Canvas)  {
    canvas.draw(
        &graphics::Quad,
        graphics::DrawParam::new()
            .dest_rect(cell_to_rect(cell))
            .color([1.,1.,1.,1.]),
        );
}

//################################################
//Supporting functions which assist generation
//################################################

fn neighbours(cell: &Cell) -> Vec<Cell> {
    let x: i32 = cell.0;
    let y: i32 = cell.1;
    vec![
    (x-1,y-1), (x,y-1), (x+1,y-1),
    (x-1,y),            (x+1,y),
    (x-1,y+1), (x,y+1), (x+1,y+1),
    ]
}

fn neighbour_counts(col: &Colony) -> DashMap<Cell, i32> {
    let ncnts = DashMap::new();

    col.par_iter().for_each( |alive_cell| {
        for neighbour in neighbours(&alive_cell) {
            *ncnts.entry(neighbour).or_insert(0) += 1;
        }
    });
    ncnts
}

/*
So the way the above code works (took me too long to figure out) is that cel.iter() makes an iterator
for the colony (all cells) which has flat_map called on it, with the function neighbors given 
as a parameter. flat_map applies the neighbors function to each cell, producing a Vec of new neighbor cells
All of the elements in these vecs are added to a single iterator by flat_map (that's how it works),
which is what "cell in" iterates through. So each cell which neighbors a live cell is visited, 
and there is redundancy. Cells which have multiple living neighbors appear more than once. once for each
neighbor in fact. And so, the .entry command adds each cell which is adjacent to a living one 
(hereby shortened to neighbor), or not, if it is already present, to the new hashmap.
.entry returns a mutable reference to the value tied to the key, which is then initialized to 0
by .or_insert(0) if needed, and then 1 is added to the value. A dereference ("*"") is needed because .or_insert
returns a mutable reference. 1 is added each time we "visit" a cell, so the result is a map of each cell
(alive or dead) with 1 or more living neighbors, and the number of living neighbors it has. You may be thinking,
"what about the cell in the middle? where is it's count?" And the answer is that if a cell has 0 neighbors,
it's gonna die so it doesn't matter lol
*/

fn generation(col: &Colony) -> Colony {
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

fn random_cells() -> Colony {
    let mut rng = rand::thread_rng();
    let mut cells: Vec<Cell> = Vec::new();
    for row in 0..TILES_SPANNING 
    {
        for column in 0..TILES_SPANNING
        {
            if rng.gen_range(0..2) == 1 {
                cells.push(((row as i32),(column as i32)));
            }
        }
    }
    let cell_set: Colony = cells.into_iter().collect();
    cell_set
}

struct MainState { //Used to contain all the information of the game while running
    screen: graphics::ScreenImage,
    col: Colony,
    frame_count: i32,
    start_time: std::time::Instant,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let screen = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb,1.,1.,1);
        //let col = HashSet::<Cell>::new();
        let col = random_cells();
        
        let state = MainState {
            screen,
            col,
            frame_count: 0,
            start_time: Instant::now(),
        };
        Ok(state)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        
        while ctx.time.check_update_time(DESIRED_FPS) {
            //let seconds = 1.0 / (DESIRED_FPS as f32);
            let new_col = generation(&self.col);
            self.col = new_col;

            //used for framerate calculation
            self.frame_count += 1;
            //break; //Very important for running below intended fps and not included in demo...
        }
        
        //Show actual framerate
        if PRINT_FPS {
            if self.frame_count >= 30 {
                let elapsed_time = self.start_time.elapsed();
                let fps = self.frame_count as f64 / elapsed_time.as_secs_f64();
                println!("fps: {:.0}", fps);
                self.start_time = Instant::now();
                self.frame_count = 0;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_screen_image(ctx, &mut self.screen, Color::BLACK);

         let time1 = self.start_time.elapsed();

        for cell in self.col.iter() {
            draw_cell(&cell, &mut canvas);
        }
        if PRINT_DRAW_TIME {
            let time2 = self.start_time.elapsed();
            let draw_time = time2 - time1;
            println!("It takes {} seconds to draw", draw_time.as_secs_f64());
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
