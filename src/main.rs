mod scene;
mod particle;
mod texture;

use scene::Scene;

use iced::time::Instant;
use iced::widget::shader::wgpu;
use iced::widget::{checkbox, column, row, shader, slider, text};
use iced::window;
use iced::{Alignment, Color, Element, Length, Subscription};

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
    Tick(Instant),
}

impl Simulation {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            scene: Scene::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ParticlesNumberChanged(amount) => {
                self.scene.change_number(amount);
            }
            Message::Tick(_time) => {
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let top_controls = row![
            control(
                "Number",
                slider(
                    1..=scene::MAX,
                    self.scene.particles.len() as u32,
                    Message::ParticlesNumberChanged
                )
                .width(100)
            ),
        ]
        .spacing(40);

        let controls = column![top_controls,]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::Center);

        let shader =
            shader(&self.scene).width(Length::Fill).height(Length::Fill);

        column![shader, controls].align_items(Alignment::Center).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

fn control<'a>(
    label: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}