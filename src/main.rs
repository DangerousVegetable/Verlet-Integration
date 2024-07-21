mod scene;
mod particle;
mod texture;
mod solver;

use scene::{Scene, MAX_FOV};

use iced::time::Instant;
use iced::widget::{column, row, shader, slider, text};
use iced::{Application};
use iced::time;
use iced::{Alignment, Element, Length, Subscription};
use iced::executor;
use iced::Theme;
use iced::Command;

use glam::{Vec2, vec2};
use smog::CustomApplication;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

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
        "TanX".to_string()
    }

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                start: Instant::now(),
                scene: Scene::new(10, solver::Constraint::Box(vec2(-40., -2.5), vec2(40., 50.))),
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
                self.scene.update(0.1);
                self.scene.update(0.1);
                self.scene.update(0.1);
                self.scene.update(0.1);
                self.scene.update(0.1);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let number_str = self.scene.simulation.particles.len().to_string();
        let number_controls = row![
            control(
                &number_str,
                slider(
                    1..=solver::MAX,
                    self.scene.simulation.particles.len() as u32,
                    |n| Message::ParticlesNumberChanged(n as usize)
                )
                .width(3000)
            ),
        ]
        .spacing(40);

        let fov_controls = row![
            control(
                "FOV",
                slider(
                    1. ..=MAX_FOV,
                    self.scene.camera.fov,
                    Message::CameraFovChanged
                )
                .width(100)
            ),
        ]
        .spacing(40);
        let x_controls = row![
            control(
                "X",
                slider(
                    -50. ..=50.,
                    self.scene.camera.pos.x,
                    Message::CameraXUpdated
                )
                .width(300)
            ),
        ]
        .spacing(40);
        let y_controls = row![
            control(
                "Y",
                slider(
                    -50. ..=50.,
                    self.scene.camera.pos.y,
                    Message::CameraYUpdated
                )
                .width(300)
            ),
        ]
        .spacing(40);

        let camera_controls = row![fov_controls, x_controls, y_controls]
            .spacing(10);
        let controls = column![number_controls, camera_controls]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::Center);

        let shader =
            shader(&self.scene).width(Length::Fill).height(Length::Fill);

        column![shader, controls]
        .align_items(Alignment::Center)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(time::Duration::from_millis(16))
            .map(Message::Tick)
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
    Tick(Instant),
}

fn control<'a>(
    label: &str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}