use crate::{
    consts::{HEIGHT, SAVEFILE, WIDTH},
    renderer::{Color, Renderer},
    sdl2_renderer::SDL2CanvasWrapper,
    vec2::Vec2,
    world::{Edge, Particle, World},
};
use sdl2::{
    event::Event,
    gfx::framerate::FPSManager,
    keyboard::{KeyboardState, Keycode, Mod, Scancode},
    mouse::{MouseButton, MouseState},
    video::{Window, WindowBuildError},
    EventPump, IntegerOrSdlError, TimerSubsystem,
};

use serde::{Deserialize, Serialize};

struct Log<const N: usize> {
    buffer: std::collections::VecDeque<String>,
}

impl<const N: usize> Log<N> {
    pub fn new() -> Self {
        Self {
            buffer: std::collections::VecDeque::with_capacity(N),
        }
    }

    pub fn log(&mut self, msg: String) {
        if self.buffer.len() == N {
            self.buffer.pop_back();
        }
        self.buffer.push_front(msg);
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &String> {
        self.buffer.iter()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum EdgePoint {
    Start,
    End,
}

#[derive(Serialize, Deserialize)]
struct State {
    world: World,
    speed: f64,
    simulate: bool,
    draw_springs: bool,
    draw_particles: bool,
}
pub struct App {
    state: State,
    timer: TimerSubsystem,
    fps_manager: FPSManager,
    canvas: SDL2CanvasWrapper<Window>,
    events: EventPump,
    fps: u8,
    rect_start: Option<Vec2>,
    line_start: Option<Vec2>,
    selected_edge: Option<(usize, EdgePoint)>,
    log: Log<10>,
    draw_log: bool,
}
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum AppConstructorError {
    CouldNotGetContext(String),
    CouldNotGetVideoSubsystem(String),
    CouldNotGetTimerSubsystem(String),
    CouldNotCreateWindow(WindowBuildError),
    CouldNotGetCanvas(IntegerOrSdlError),
    CouldNotGetEventPump(String),
    CouldNotSetFPS(String),
}

impl std::fmt::Display for AppConstructorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppConstructorError::CouldNotGetContext(msg) => {
                f.write_fmt(format_args!("could not get SDL context: {msg}"))
            }
            AppConstructorError::CouldNotGetVideoSubsystem(msg) => {
                f.write_fmt(format_args!("could not get video subsystem: {msg}"))
            }
            AppConstructorError::CouldNotGetTimerSubsystem(msg) => {
                f.write_fmt(format_args!("could not get timer subsystem: {msg}"))
            }
            AppConstructorError::CouldNotCreateWindow(msg) => {
                f.write_fmt(format_args!("could not construct window: {msg}"))
            }
            AppConstructorError::CouldNotGetCanvas(msg) => {
                f.write_fmt(format_args!("could not get canvas from window: {msg}"))
            }
            AppConstructorError::CouldNotGetEventPump(msg) => {
                f.write_fmt(format_args!("could not get event pump {msg}"))
            }
            AppConstructorError::CouldNotSetFPS(msg) => {
                f.write_fmt(format_args!("could not set fps: {msg}"))
            }
        }
    }
}

impl std::error::Error for AppConstructorError {}

impl App {
    pub fn new() -> Result<Self, AppConstructorError> {
        let ctx = sdl2::init().map_err(AppConstructorError::CouldNotGetContext)?;
        let video = ctx
            .video()
            .map_err(AppConstructorError::CouldNotGetVideoSubsystem)?;
        let window = video
            .window("soft", WIDTH as u32, HEIGHT as u32)
            .fullscreen()
            .build()
            .map_err(AppConstructorError::CouldNotCreateWindow)?;
        let canvas = window
            .into_canvas()
            .build()
            .map_err(AppConstructorError::CouldNotGetCanvas)?
            .into();
        let timer = ctx
            .timer()
            .map_err(AppConstructorError::CouldNotGetTimerSubsystem)?;
        let events = ctx
            .event_pump()
            .map_err(AppConstructorError::CouldNotGetEventPump)?;

        let mut app = App {
            state: State {
                world: World::new(),
                speed: 1.0,
                simulate: false,
                draw_springs: false,
                draw_particles: false,
            },
            timer,
            fps_manager: FPSManager::new(),
            canvas,
            events,
            fps: 0,
            rect_start: None,
            line_start: None,
            selected_edge: None,
            log: Log::new(),
            draw_log: true,
        };

        app.fps_manager
            .set_framerate(60)
            .map_err(AppConstructorError::CouldNotSetFPS)?;

        Ok(app)
    }

    #[allow(unused_must_use)]
    pub fn init_default_world(&mut self) {
        let world = &mut self.state.world;

        world.add_edge(Vec2::new(0.0, 400.0), Vec2::new(280.0, 400.0));
        world.add_edge(Vec2::new(400.0, 700.0), Vec2::new(680.0, 700.0));

        world.add_edge(Vec2::new(850.0, 1080.0), Vec2::new(1920.0, 800.0));

        // Screen edges
        world.add_edge(
            Vec2::new(-Edge::R, -Edge::R),
            Vec2::new(-Edge::R, 1080.0 + Edge::R),
        );
        world.add_edge(
            Vec2::new(-Edge::R, -Edge::R),
            Vec2::new(1920.0 + Edge::R, -Edge::R),
        );
        world.add_edge(
            Vec2::new(-Edge::R, 1080.0 + Edge::R),
            Vec2::new(1920.0 + Edge::R, 1080.0 + Edge::R),
        );
        world.add_edge(
            Vec2::new(1920.0 + Edge::R, -Edge::R),
            Vec2::new(1920.0 + Edge::R, 1080.0 + Edge::R),
        );
    }

    pub fn load_or_default(&mut self) {
        match std::fs::read_to_string(SAVEFILE) {
            Ok(save) => {
                let msg = if let Ok(state) = serde_json::from_str(save.as_str()) {
                    self.load_state(state);
                    "savefile loaded succesfully"
                } else {
                    self.init_default_world();
                    "could not deserialize savefile"
                };

                self.log.log(msg.into());
            }
            Err(_) => self.init_default_world(),
        }
    }

    fn load_state(&mut self, state: State) {
        self.state = state;
        self.selected_edge = None;
    }

    fn save_state(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.state)
    }

    #[allow(clippy::too_many_lines)]
    fn handle_events(&mut self) -> bool {
        let lctrl = self
            .events
            .keyboard_state()
            .is_scancode_pressed(Scancode::LCtrl);

        let events: Vec<Event> = self.events.poll_iter().collect();
        for event in events {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.state.simulate = !self.state.simulate;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => {
                    self.state.draw_particles = !self.state.draw_particles;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => {
                    self.state.draw_springs = !self.state.draw_springs;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F3),
                    ..
                } => {
                    self.draw_log = !self.draw_log;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F4),
                    ..
                } => {
                    let msg = match std::fs::write(
                        SAVEFILE,
                        self.save_state().expect("state should be valid to save"),
                    ) {
                        Ok(_) => format!("world saved to {SAVEFILE}"),
                        Err(err) => format!("Could not save file: {err}"),
                    };

                    self.log.log(msg);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } => match std::fs::read_to_string(SAVEFILE) {
                    Ok(save) => {
                        let msg = if let Ok(state) = serde_json::from_str(save.as_str()) {
                            self.load_state(state);
                            "savefile loaded succesfully"
                        } else {
                            "could not deserialize savefile"
                        };

                        self.log.log(msg.into());
                    }
                    Err(err) => self.log.log(format!("could not open savefile: {err}")),
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } if self.state.speed > 0.0 => {
                    self.state.speed -= 0.01;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } if self.state.speed < 2.0 => {
                    self.state.speed += 0.01;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    self.state.world.remove_last();
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    if lctrl {
                        self.line_start = Some(Vec2::new(f64::from(x), f64::from(y)));
                        self.rect_start = None;
                    } else {
                        self.rect_start = Some(Vec2::new(f64::from(x), f64::from(y)));
                        self.line_start = None;
                    }
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } if self.rect_start.is_some() => {
                    if let Err((w, h)) = self.state.world.spawn_rect(
                        ((self.rect_start.unwrap().x - f64::from(x)).abs() / Particle::SPACING)
                            as usize
                            + 1,
                        ((self.rect_start.unwrap().y - f64::from(y)).abs() / Particle::SPACING)
                            as usize
                            + 1,
                        f64::min(self.rect_start.unwrap().x, f64::from(x)),
                        f64::min(self.rect_start.unwrap().y, f64::from(y)),
                    ) {
                        self.log.log(format!(
                            "error while spawning new rect: Rect is too small: ({w}, {h}) < (2, 2)"
                        ));
                    }

                    self.rect_start = None;
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } if self.line_start.is_some() => {
                    if let Err(msg) = self.state.world.add_edge(
                        self.line_start.unwrap(),
                        Vec2::new(f64::from(x), f64::from(y)),
                    ) {
                        self.log.log(msg.into());
                    }
                    self.line_start = None;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Delete),
                    keymod: Mod::NOMOD,
                    ..
                } => {
                    self.state.world.clear();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Delete),
                    keymod: Mod::LCTRLMOD,
                    ..
                } => {
                    if let Some((n, _)) = self.selected_edge {
                        self.state.world.remove_edge(n);
                        self.selected_edge = None;
                    }
                }

                _ => {}
            }
        }
        true
    }

    pub fn run(mut self) {
        'running: loop {
            let (begin, mouse, _) = self.begin_frame();

            if !self.handle_events() {
                break 'running;
            }

            if self.state.simulate {
                self.update_physics();
            }

            self.draw_world();
            self.draw_ui();

            let mouse_pos = Vec2::new(f64::from(mouse.x()), f64::from(mouse.y()));

            self.handle_new_rect(mouse_pos);
            self.handle_new_line(mouse_pos);
            self.handle_line_manip(mouse, mouse_pos);

            self.end_frame(begin);
        }
    }

    fn handle_line_manip(&mut self, mouse: MouseState, mouse_pos: Vec2) {
        if let Some((n, which_end)) = self.selected_edge {
            let e = self
                .state
                .world
                .edges_iter_mut()
                .nth(n)
                .expect("Index of edge should always be valid");

            match which_end {
                EdgePoint::Start => {
                    self.canvas
                        .set_color(Color::CYAN)
                        .filled_circle(e.get_start(), Edge::R);

                    if mouse.is_mouse_button_pressed(MouseButton::Left) {
                        e.set_start(mouse_pos);
                    } else {
                        self.selected_edge = None;
                    }
                }
                EdgePoint::End => {
                    self.canvas
                        .set_color(Color::CYAN)
                        .filled_circle(e.get_end(), Edge::R);

                    if mouse.is_mouse_button_pressed(MouseButton::Left) {
                        e.set_end(mouse_pos);
                    } else {
                        self.selected_edge = None;
                    }
                }
            };
        }
        //FIXME: This snippet must go after the previous. fix this.
        let mut itr = self.state.world.edges_iter().enumerate();
        while self.selected_edge.is_none() && let Some((i,e)) = itr.next() {

            if Vec2::dist_sqr(e.get_start(), mouse_pos) < Edge::R * Edge::R {
                self.selected_edge = Some((i, EdgePoint::Start));
            } else if Vec2::dist_sqr(e.get_end(), mouse_pos) < Edge::R * Edge::R {
                self.selected_edge = Some((i, EdgePoint::End));
            }
        }
    }

    fn handle_new_line(&mut self, mouse_pos: Vec2) {
        if let Some(start_pos) = self.line_start {
            if self.state.world.can_add_edge(start_pos, mouse_pos) {
                self.canvas.set_color(Color::RGB(44, 56, 80));
            } else {
                self.canvas.set_color(Color::RED);
            };
            self.canvas
                .thick_line(start_pos, mouse_pos, Edge::R * 2.0)
                .set_color(Color::RGB(88, 112, 161))
                .filled_circle(start_pos, Edge::R)
                .filled_circle(mouse_pos, Edge::R);
        }
    }

    fn handle_new_rect(&mut self, mouse_pos: Vec2) {
        if let Some(start_pos) = self.rect_start {
            let size = (Vec2::abs_diff(start_pos, mouse_pos) / Particle::SPACING).ceil();

            if self
                .state
                .world
                .can_spawn_rect(size.x as usize, size.y as usize)
            {
                self.canvas.set_color(Color::RGB(44, 56, 80));
            } else {
                self.canvas.set_color(Color::RED);
            };

            self.canvas.rectangle(start_pos, mouse_pos);

            self.canvas.text(
                start_pos + Vec2::new(10.0, -10.0),
                format!("{:.0} x {:.0}", size.x, size.y).as_str(),
            );
        }
    }

    fn update_physics(&mut self) {
        if let Err(diff_len) = self.state.world.update() {
            self.log.log(format!(
                "suspiciously large spring strech detected. diff_len={diff_len}. World reset."
            ));
            self.state.world.clear();
        }
    }

    fn begin_frame(&mut self) -> (u32, MouseState, KeyboardState) {
        self.canvas.set_color(Color::RGB(11, 14, 20));
        self.canvas.clear();

        let begin = self.timer.ticks();
        let mouse = self.events.mouse_state();
        let keyboard = self.events.keyboard_state();
        (begin, mouse, keyboard)
    }

    fn draw_world(&mut self) {
        if self.state.draw_springs {
            self.state.world.draw_springs(&mut self.canvas);
        }
        if self.state.draw_particles {
            self.state.world.draw_particles(&mut self.canvas);
        }
        if !(self.state.draw_particles || self.state.draw_springs) {
            self.state.world.draw_polys(&mut self.canvas);
        }
        self.state.world.draw_edges(&mut self.canvas);
    }

    fn end_frame(&mut self, begin: u32) {
        self.canvas.finish();
        self.fps_manager.delay();

        let frame_time = f64::from(self.timer.ticks() - begin);
        self.fps = (1000.0 / frame_time) as u8;

        if self.state.simulate {
            self.state
                .world
                .end_frame(self.state.speed * (frame_time / 1000.0));
        }
    }

    fn draw_ui(&mut self) {
        let (p_len, s_len, b_len, e_len, o_len) = self.state.world.info();

        self.canvas
            .set_color(Color::RGBA(88, 112, 160, 120))
            .filled_rounded_rectangle(Vec2::new(15.0, 15.0), Vec2::new(145.0, 110.0), 5.0)
            .set_color(Color::CYAN)
            .text(Vec2::new(20.0, 25.0), format!("{} FPS", self.fps).as_str())
            .set_color(Color::RGB(176, 224, 255))
            .text(Vec2::new(20.0, 40.0), format!("{p_len} particles").as_str())
            .text(Vec2::new(20.0, 50.0), format!("{s_len} springs").as_str())
            .text(
                Vec2::new(20.0, 60.0),
                format!("{b_len} boundaries").as_str(),
            )
            .text(Vec2::new(20.0, 70.0), format!("{e_len} edges").as_str())
            .text(Vec2::new(20.0, 80.0), format!("{o_len} objects").as_str());

        let spd = if self.state.simulate {
            format!("speed: {:.2}x", self.state.speed)
        } else {
            String::from("paused")
        };
        self.canvas.text(Vec2::new(20.0, 90.0), spd.as_str());

        if self.draw_log && self.log.len() != 0 {
            self.canvas
                .set_color(Color::RGBA(88, 112, 160, 120))
                .filled_rounded_rectangle(
                    Vec2::new(385.0, 5.0),
                    Vec2::new(WIDTH - 385.0, 9.0 * 15.0),
                    5.0,
                )
                .set_color(Color::RGB(176, 224, 255));
            for (i, msg) in self.log.iter().rev().enumerate() {
                self.canvas
                    .text(Vec2::new(400.0, 15.0 + 10.0 * i as f64), msg.as_str());
            }
        }
    }
}
