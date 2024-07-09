mod scene;
mod particle;
mod texture;
mod solver;

use core::num;

use iced::widget::shader::wgpu::Adapter;
use scene::Scene;

use iced::time::Instant;
use iced::widget::{column, row, shader, slider, text};
use iced::{window, Settings};
use iced::time;
use iced::{Alignment, Element, Length, Subscription};

use glam::{Vec2, vec2};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::program(
        "TanX - the Game",
        Simulation::update,
        Simulation::view,
    )
    .subscription(Simulation::subscription)
    .run()
}

struct Simulation {
    start: Instant,
    scene: Scene,
}

#[derive(Debug, Clone)]
enum Message {
    ParticlesNumberChanged(u32),
    CameraFovChanged(f32),
    CameraXUpdated(f32),
    CameraYUpdated(f32),
    Tick(Instant),
}

impl Simulation {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            scene: Scene::new(10, solver::Constraint::Box(vec2(-20., -2.5), vec2(20., 50.))),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ParticlesNumberChanged(amount) => {
                self.scene.change_number(amount);
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
                self.scene.update(0.05);
                self.scene.update(0.05);
                self.scene.update(0.05);
                self.scene.update(0.05);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let number_str = self.scene.particles.len().to_string();
        let number_controls = row![
            control(
                &number_str,
                slider(
                    1..=scene::MAX,
                    self.scene.particles.len() as u32,
                    Message::ParticlesNumberChanged
                )
                .width(1000)
            ),
        ]
        .spacing(40);

        let fov_controls = row![
            control(
                "FOV",
                slider(
                    1. ..=100.,
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
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

fn control<'a>(
    label: &str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}