mod simulation;

use glium::*;
use glutin::*;
use handle_events::*;

use std::time::{Duration, Instant};
use std::ops::Deref;
use self::simulation::*;

pub enum GameFn {
    StateFn(fn(&mut GameStruct) -> GameFn),
    Exit,
}

impl GameFn {
    pub fn new(function: fn(&mut GameStruct) -> GameFn) -> GameFn {
        GameFn::StateFn(function)
    }
    pub fn exit() -> GameFn {
        GameFn::Exit
    }
    pub fn running(&self) -> bool {
        match self {
            &GameFn::Exit => false,
            _ => true,
        }
    }
}

impl Deref for GameFn {
    type Target = fn(&mut GameStruct) -> GameFn;
    fn deref(&self) -> &Self::Target {
        match self {
            &GameFn::StateFn(ref func) => func,
            &GameFn::Exit => {
                panic!("attempt to call game-fn in exit state");
            }
        }
    }
}

/// Represents resources that are available through all game-states. Implemented as a state-machine
/// of separate game states.
pub struct GameStruct {
    display: Display,
    events_loop: EventsLoop,
    fps_calc: FpsCalculator,
    print_timer: f32,
}

const PRINT_INTERVAL: f32 = 2f32;
impl GameStruct {
    pub fn new(events_loop: EventsLoop, display: Display) -> GameStruct {
        GameStruct {
            display,
            events_loop,
            fps_calc: FpsCalculator::new(),
            print_timer: PRINT_INTERVAL,
        }
    }

    /// Initialize game-state.
    pub fn init(&mut self) -> GameFn {
        info!("GameStruct::init");

        // After initialization, go to simulation
        GameFn::new(Self::simulation)
    }

    /// The game.
    pub fn simulation(&mut self) -> GameFn {
        info!("GameStruct::simulation");

        const FIXED_TIMESTEP_NS: u32 = 16666667;
        let fixed_deltatime = Duration::new(0, FIXED_TIMESTEP_NS);
        let fixed_timestep_s = (fixed_deltatime.as_secs() as f64
            + fixed_deltatime.subsec_nanos() as f64 * 1e-9) as f32;
        debug!(
            "fixed-timestep: {:.2} ms, fixed-FPS: {:.1}",
            fixed_timestep_s * 1e+3,
            1f32 / fixed_timestep_s
        );

        let mut simulation = Simulation::new(&mut self.display);

        // Update eg. camera before starting the main loop
        simulation.late_update(&mut self.display);

        // Fixed delta-time accumulator
        let mut fdt_accumulator = Duration::new(0, 0);
        let mut last_frame_time = Instant::now();
        loop {
            // Render game
            simulation.draw(&mut self.display);

            // Collect events from window and devices
            let user_actions = poll_events(&mut self.events_loop);

            // Handle eg. resizing and exiting window
            if !simulation.process_events(&user_actions) {
                return GameFn::exit();
            }

            let now = Instant::now();
            let dt = now - last_frame_time;
            fdt_accumulator += dt;
            last_frame_time = now;

            while fdt_accumulator >= fixed_deltatime {
                fdt_accumulator -= fixed_deltatime;

                let fixed_dt = (fixed_deltatime.as_secs() as f64
                    + fixed_deltatime.subsec_nanos() as f64 * 1e-9)
                    as f32;

                simulation.fixed_update(fixed_dt);
            }
            let dt = (dt.as_secs() as f64 + dt.subsec_nanos() as f64 * 1e-9) as f32;
            simulation.update(dt);

            // Measure fps
            self.fps_calc.store_dt(dt);

            // Print diagnostics periodically
            self.print_timer -= dt;
            if self.print_timer <= 0f32 {
                self.print_timer += PRINT_INTERVAL;
                debug!("FPS: {:.1}", self.fps_calc.fps());
            }

            // Update eg. camera
            simulation.late_update(&mut self.display);

            // TODO: Go to game-end state after the game has finished
            if simulation.ended() {
                return GameFn::exit();
            }
        }
    }
}

const SAMPLE_SIZE: usize = 10;
/// Counts the running average delta-time and FPS.
struct FpsCalculator {
    dt_buffer: [f32; SAMPLE_SIZE],
    it: usize,
}

impl FpsCalculator {
    pub fn new() -> FpsCalculator {
        FpsCalculator {
            dt_buffer: [0f32; SAMPLE_SIZE],
            it: 0,
        }
    }
    pub fn store_dt(&mut self, dt: f32) {
        self.dt_buffer[self.it] = dt;
        self.it += 1;
        if self.it == SAMPLE_SIZE {
            self.it = 0;
        }
    }
    pub fn fps(&self) -> f32 {
        SAMPLE_SIZE as f32 / self.dt_buffer.iter().sum::<f32>()
    }
}
