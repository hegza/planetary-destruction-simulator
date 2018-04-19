use glutin;
use util::camera::*;

#[derive(Debug)]
pub enum Action {
    Exit,
    Refresh,
    SetAspect(f32),
    /// Rotate control clockwise ("left")
    CamRotateCw(bool),
    /// Rotate control counter-clockwise ("right")
    CamRotateCcw(bool),
    /// Rotate control north
    CamRotateN(bool),
    /// Rotate control south
    CamRotateS(bool),
    /// Shoot laser
    Shoot(bool),
}

pub fn poll_events(events_loop: &mut glutin::EventsLoop) -> Vec<Action> {
    let mut actions = Vec::new();
    use self::Action::*;
    events_loop.poll_events(|event| match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            glutin::WindowEvent::Closed => {
                actions.push(Exit);
            }
            glutin::WindowEvent::KeyboardInput { input, .. } => {
                if let Some(vk) = input.virtual_keycode {
                    let set = match input.state {
                        glutin::ElementState::Pressed => true,
                        glutin::ElementState::Released => false,
                    };
                    use glutin::VirtualKeyCode as VK;
                    match vk {
                        VK::Left => actions.push(CamRotateCw(set)),
                        VK::Right => actions.push(CamRotateCcw(set)),
                        VK::Up => actions.push(CamRotateN(set)),
                        VK::Down => actions.push(CamRotateS(set)),
                        // Ctrl + R -> restart simulation
                        VK::R => {
                            if set && input.modifiers.ctrl {
                                actions.push(Refresh);
                            }
                        }
                        VK::Space => {
                            let set = match input.state {
                                glutin::ElementState::Pressed => true,
                                glutin::ElementState::Released => false,
                            };
                            actions.push(Shoot(set));
                        },
                        _ => {}
                    }
                }
            }
            glutin::WindowEvent::Resized(w, h) => {
                actions.push(SetAspect(w as f32 / h as f32));
            }
            _ => {}
        },
        _ => {}
    });
    actions
}

#[derive(Debug)]
pub enum ProgramCommand {
    Exit,
    RefreshSimulation,
}

pub fn process_global_events(camera: &mut Camera, actions: &[Action]) -> Option<ProgramCommand> {
    let mut command = None;
    actions.iter().for_each(|action| {
        use self::Action::*;
        match *action {
            Exit => {
                command = Some(ProgramCommand::Exit);
            }
            SetAspect(naspect) => {
                camera.set_aspect(naspect);
            }
            Refresh => command = Some(ProgramCommand::RefreshSimulation),
            _ => {}
        }
    });
    command
}

pub fn process_camera_events(control: &mut CameraControl, actions: &[Action]) {
    actions.iter().for_each(|action| {
        use self::Action::*;
        match *action {
            CamRotateCw(set) => control.cw = set,
            CamRotateCcw(set) => control.ccw = set,
            CamRotateN(set) => control.n = set,
            CamRotateS(set) => control.s = set,
            _ => {}
        }
    });
}
