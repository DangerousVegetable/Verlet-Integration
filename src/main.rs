mod particle;
mod scene;
mod solver;
mod texture;

use iced_core::SmolStr;
use scene::{Scene, MAX_FOV};

use iced::executor;
use iced::time;
use iced::time::Instant;
use iced::widget::{column, row, shader, slider, text};
use iced::Command;
use iced::Theme;
use iced::{event, keyboard, Application};
use iced::{Alignment, Element, Length, Subscription};

use glam::{vec2, Vec2};
use verlet_integration::CustomApplication;

const SUB_TICKS: usize = 8;
fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    //rayon::ThreadPoolBuilder::new()
    //    .num_threads(10)
    //    .build_global()
    //    .unwrap();

    <Simulation as CustomApplication>::run(iced::Settings::default())
}

struct Simulation {
    start: Instant,
    scene: Scene,
}

impl CustomApplication for Simulation {}

impl Application for Simulation {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn title(&self) -> String {
        "Verlet PhysX Engine".to_string()
    }

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                start: Instant::now(),
                scene: Scene::new(
                    10,
                    solver::Constraint::Box(vec2(-60., -10.), vec2(60., 40.)),
                ),
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::ParticlesNumberChanged(number) => {
                self.scene.change_number(number);
            }
            Message::CameraFovChanged(fov) => {
                self.scene.camera.fov = fov;
            }
            Message::CameraXUpdated(x) => {
                self.scene.camera.pos.x = x;
            }
            Message::CameraYUpdated(y) => {
                self.scene.camera.pos.y = y;
            }
            Message::Tick(_time) => {
                let time = Instant::now();
                let dt = 0.08/SUB_TICKS as f32;
                for _ in 0..SUB_TICKS {
                    self.scene.update(dt);
                }
                //println!("{}", (Instant::now() - time).as_nanos() as f32 / 1000000.);
            }
            Message::Event(event) => match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Space),
                    location: _,
                    modifiers: _,
                    text: _,
                }) => {
                    self.scene.change_number(self.scene.simulation.particles.len() + 100);
                }
                _ => {}
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let number_str = self.scene.simulation.particles.len().to_string();
        let number_controls = row![control(
            &number_str,
            slider(
                1..=solver::MAX,
                self.scene.simulation.particles.len() as u32,
                |n| Message::ParticlesNumberChanged(n as usize)
            )
            .width(3000)
        ),]
        .spacing(40);

        let fov_controls = row![control(
            "FOV",
            slider(
                1. ..=MAX_FOV,
                self.scene.camera.fov,
                Message::CameraFovChanged
            )
            .width(100)
        ),]
        .spacing(40);
        let x_controls = row![control(
            "X",
            slider(
                -50. ..=50.,
                self.scene.camera.pos.x,
                Message::CameraXUpdated
            )
            .width(300)
        ),]
        .spacing(40);
        let y_controls = row![control(
            "Y",
            slider(
                -50. ..=50.,
                self.scene.camera.pos.y,
                Message::CameraYUpdated
            )
            .width(300)
        ),]
        .spacing(40);

        let camera_controls = row![fov_controls, x_controls, y_controls].spacing(10);
        let controls = column![number_controls, camera_controls]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::Center);

        let shader = shader(&self.scene).width(Length::Fill).height(Length::Fill);

        column![shader, controls]
            .align_items(Alignment::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(time::Duration::from_millis(16)).map(Message::Tick),
            event::listen().map(Message::Event),
        ])
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

#[derive(Debug, Clone)]
enum Message {
    ParticlesNumberChanged(usize),
    CameraFovChanged(f32),
    CameraXUpdated(f32),
    CameraYUpdated(f32),
    Event(iced::Event),
    Tick(Instant),
}

fn control<'a>(label: &str, control: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}
