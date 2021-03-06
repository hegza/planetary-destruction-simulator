extern crate cgmath;
extern crate config;
extern crate env_logger;
extern crate genmesh;
#[macro_use]
extern crate glium;
extern crate glutin;
extern crate image;
extern crate isosurface;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate ndarray;
extern crate obj;
extern crate ocl;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

mod game;
mod init;
mod handle_events;
mod prelude;
mod shader;
mod util;

use glium::*;
use game::*;

fn main() {
    // Init logging
    env_logger::init();

    // Init context
    let events_loop = glutin::EventsLoop::new();
    let viewport = Rect {
        left: 0,
        bottom: 0,
        width: 1024,
        height: 768,
    };
    let display = init::open_display("Planetary destruction simulator", viewport, &events_loop);

    // Init game state
    let mut game = GameStruct::new(events_loop, display);

    // Run game
    let mut current = GameFn::new(GameStruct::init);
    loop {
        current = current(&mut game);
        if !current.running() {
            break;
        }
    }
}
