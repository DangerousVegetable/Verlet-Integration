//! An animated solar system.
//!
//! This example showcases how to use a `Canvas` widget with transforms to draw
//! using different coordinate systems.
//!
//! Inspired by the example found in the MDN docs[1].
//!
//! [1]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_animations#An_animated_solar_system

use iced::mouse;
use iced::time;
use iced::widget::canvas;
use iced::widget::canvas::gradient;
use iced::widget::canvas::stroke::{self, Stroke};
use iced::widget::canvas::{Geometry, Path};
use iced::widget::shader::Primitive;
use iced::window;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Subscription, Theme, Vector};

use std::time::Instant;

use std::time::Duration;

mod particle;
mod texture;
mod scene;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::program("Verlet PhysX", Simulation::update, Simulation::view)
        .subscription(Simulation::subscription)
        .antialiasing(false)
        .theme(Simulation::theme)
        .run()
}

#[derive(Default)]
struct Simulation {
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

impl Simulation {
    const PHYS_TICK: u64 = 10;

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => self.state.update(),
        }
    }

    fn view(&self) -> Element<Message> {
        canvas(&self.state)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        //window::frames().map(Message::Tick)
        time::every(Duration::from_millis(Self::PHYS_TICK)).map(|_| Message::Tick)
    }
}

#[derive(Debug)]
struct State {
    canvas_cache: canvas::Cache,
    tick: u128,
    particles: Vec<Particle>,
}

impl State {
    pub fn new() -> State {
        let size = window::Settings::default().size;

        State {
            canvas_cache: canvas::Cache::default(),
            tick: 0,
            particles: Self::generate_particles(20000, size.width, size.height),
        }
    }

    pub fn update(&mut self) {
        self.canvas_cache.clear();
        self.tick += 1;
        dbg!(self.tick);
    }

    fn generate_particles(num: i32, width: f32, height: f32) -> Vec<Particle> {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        (0..num)
            .map(|_| {
                Particle::new(
                    rng.gen_range((-width / 2.0)..(width / 2.0)),
                    rng.gen_range((-height / 2.0)..(height / 2.0)),
                )
            })
            .collect()
    }
}

impl<Message> canvas::Program<Message> for State {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let system = self.canvas_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            frame.translate(Vector::new(center.x, center.y));
            for &p in &self.particles {
                // let path = canvas::Path::circle(p.into(), Particle::RADIUS);
                frame.fill_rectangle(
                    p.into(),
                    Size::new(Particle::RADIUS, Particle::RADIUS),
                    Color::WHITE,
                );
                //frame.fill(&path, Color::WHITE);
            }
        });
        vec![system]
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
