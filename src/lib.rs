use iced::Application;
use iced::Settings;
use iced_wgpu::Settings as CompositorSettings;

pub trait CustomApplication: Application {
    fn run(settings: Settings<Self::Flags>) -> iced::Result
    where
        Self: 'static,
    {
        let renderer_settings = CompositorSettings {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            antialiasing: None,
            //present_mode: iced::widget::shader::wgpu::PresentMode::AutoVsync,
            ..CompositorSettings::default()
        };

        Ok(iced_winit::application::run::<
            Instance<Self>,
            Self::Executor,
            compositor::Compositor,
        >(settings.into(), renderer_settings)?)
    }
}

struct Instance<A: Application>(A);

impl<A> iced_runtime::Program for Instance<A>
where
    A: Application,
{
    type Message = A::Message;
    type Theme = A::Theme;
    type Renderer = iced::Renderer;

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        self.0.update(message)
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        self.0.view()
    }
}

impl<A> iced_winit::Application for Instance<A>
where
    A: Application,
{
    type Flags = A::Flags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<A::Message>) {
        let (app, command) = A::new(flags);

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn theme(&self) -> A::Theme {
        self.0.theme()
    }

    fn style(&self) -> <A::Theme as iced_style::application::StyleSheet>::Style {
        self.0.style()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        self.0.subscription()
    }

    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }
}

mod compositor {
    use iced_wgpu::core::Color;
    use iced_wgpu::graphics::{self, color, compositor, Error, Viewport};
    use iced_wgpu::{Backend, Primitive, Renderer, Settings};
    use iced_wgpu::wgpu;

    /// A window graphics backend for iced powered by `wgpu`.
    #[allow(missing_debug_implementations)]
    pub struct Compositor {
        settings: Settings,
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        alpha_mode: wgpu::CompositeAlphaMode,
    }

    impl Compositor {
        /// Requests a new [`Compositor`] with the given [`Settings`].
        ///
        /// Returns `None` if no compatible graphics adapter could be found.
        pub async fn request<W: compositor::Window>(
            settings: Settings,
            compatible_window: Option<W>,
        ) -> Option<Self> {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: settings.internal_backend,
                ..Default::default()
            });

            log::info!("{settings:#?}");

            #[cfg(not(target_arch = "wasm32"))]
            if log::max_level() >= log::LevelFilter::Info {
                let available_adapters: Vec<_> = instance
                    .enumerate_adapters(settings.internal_backend)
                    .iter()
                    .map(wgpu::Adapter::get_info)
                    .collect();
                log::info!("Available adapters: {available_adapters:#?}");
            }

            #[allow(unsafe_code)]
            let compatible_surface = compatible_window
                .and_then(|window| instance.create_surface(window).ok());

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::util::power_preference_from_env()
                        .unwrap_or(if settings.antialiasing.is_none() {
                            wgpu::PowerPreference::LowPower
                        } else {
                            wgpu::PowerPreference::HighPerformance
                        }),
                    compatible_surface: compatible_surface.as_ref(),
                    force_fallback_adapter: false,
                })
                .await?;

            log::info!("Selected: {:#?}", adapter.get_info());

            let (format, alpha_mode) =
                compatible_surface.as_ref().and_then(|surface| {
                    let capabilities = surface.get_capabilities(&adapter);

                    let mut formats = capabilities.formats.iter().copied();

                    log::info!("Available formats: {formats:#?}");

                    let format = if color::GAMMA_CORRECTION {
                        formats.find(wgpu::TextureFormat::is_srgb)
                    } else {
                        formats.find(|format| !wgpu::TextureFormat::is_srgb(format))
                    };

                    let format = format.or_else(|| {
                        log::warn!("No format found!");

                        capabilities.formats.first().copied()
                    });

                    let alpha_modes = capabilities.alpha_modes;

                    log::info!("Available alpha modes: {alpha_modes:#?}");

                    let preferred_alpha = if alpha_modes
                        .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
                    {
                        wgpu::CompositeAlphaMode::PostMultiplied
                    } else {
                        wgpu::CompositeAlphaMode::Auto
                    };

                    format.zip(Some(preferred_alpha))
                })?;

            log::info!(
                "Selected format: {format:?} with alpha mode: {alpha_mode:?}"
            );

            #[cfg(target_arch = "wasm32")]
            let limits = [wgpu::Limits::downlevel_webgl2_defaults()
                .using_resolution(adapter.limits())];

            #[cfg(not(target_arch = "wasm32"))]
            let limits =
                [wgpu::Limits::default(), wgpu::Limits::downlevel_defaults()];

            let mut limits = limits.into_iter().map(|limits| wgpu::Limits {
                max_bind_groups: 2,
                ..limits
            });

            let (device, queue) =
                loop {
                    let required_limits = limits.next()?;
                    let device = adapter.request_device(
                        &wgpu::DeviceDescriptor {
                            label: Some(
                                "iced_wgpu::window::compositor device descriptor",
                            ),
                            required_features: wgpu::Features::TEXTURE_BINDING_ARRAY |
                                wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                            required_limits,
                        },
                        None,
                    ).await.ok();

                    if let Some(device) = device {
                        break Some(device);
                    }
                }?;

            Some(Compositor {
                instance,
                settings,
                adapter,
                device,
                queue,
                format,
                alpha_mode,
            })
        }

        /// Creates a new rendering [`Backend`] for this [`Compositor`].
        pub fn create_backend(&self) -> Backend {
            Backend::new(&self.device, &self.queue, self.settings, self.format)
        }
    }

    /// Creates a [`Compositor`] and its [`Backend`] for the given [`Settings`] and
    /// window.
    pub fn new<W: compositor::Window>(
        settings: Settings,
        compatible_window: W,
    ) -> Result<Compositor, Error> {
        let compositor = futures::executor::block_on(Compositor::request(
            settings,
            Some(compatible_window),
        ))
        .ok_or(Error::GraphicsAdapterNotFound)?;

        Ok(compositor)
    }

    /// Presents the given primitives with the given [`Compositor`] and [`Backend`].
    pub fn present<T: AsRef<str>>(
        compositor: &mut Compositor,
        backend: &mut Backend,
        surface: &mut wgpu::Surface<'static>,
        primitives: &[Primitive],
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), compositor::SurfaceError> {
        match surface.get_current_texture() {
            Ok(frame) => {
                let mut encoder = compositor.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("iced_wgpu encoder"),
                    },
                );

                let view = &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                backend.present(
                    &compositor.device,
                    &compositor.queue,
                    &mut encoder,
                    Some(background_color),
                    frame.texture.format(),
                    view,
                    primitives,
                    viewport,
                    overlay,
                );

                // Submit work
                let _submission = compositor.queue.submit(Some(encoder.finish()));
                frame.present();

                Ok(())
            }
            Err(error) => match error {
                wgpu::SurfaceError::Timeout => {
                    Err(compositor::SurfaceError::Timeout)
                }
                wgpu::SurfaceError::Outdated => {
                    Err(compositor::SurfaceError::Outdated)
                }
                wgpu::SurfaceError::Lost => Err(compositor::SurfaceError::Lost),
                wgpu::SurfaceError::OutOfMemory => {
                    Err(compositor::SurfaceError::OutOfMemory)
                }
            },
        }
    }

    impl graphics::Compositor for Compositor {
        type Settings = Settings;
        type Renderer = iced::Renderer;
        type Surface = wgpu::Surface<'static>;

        fn new<W: compositor::Window>(
            settings: Self::Settings,
            compatible_window: W,
        ) -> Result<Self, Error> {
            new(settings, compatible_window)
        }

        fn create_renderer(&self) -> Self::Renderer {
            iced::Renderer::Wgpu(Renderer::new(
                self.create_backend(),
                self.settings.default_font,
                self.settings.default_text_size,
            ))
        }

        fn create_surface<W: compositor::Window>(
            &mut self,
            window: W,
            width: u32,
            height: u32,
        ) -> Self::Surface {
            let mut surface = self
                .instance
                .create_surface(window)
                .expect("Create surface");

            if width > 0 && height > 0 {
                self.configure_surface(&mut surface, width, height);
            }

            surface
        }

        fn configure_surface(
            &mut self,
            surface: &mut Self::Surface,
            width: u32,
            height: u32,
        ) {
            surface.configure(
                &self.device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: self.format,
                    present_mode: self.settings.present_mode,
                    width,
                    height,
                    alpha_mode: self.alpha_mode,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );
        }

        fn fetch_information(&self) -> compositor::Information {
            let information = self.adapter.get_info();

            compositor::Information {
                adapter: information.name,
                backend: format!("{:?}", information.backend),
            }
        }

        fn present<T: AsRef<str>>(
            &mut self,
            renderer: &mut Self::Renderer,
            surface: &mut Self::Surface,
            viewport: &Viewport,
            background_color: Color,
            overlay: &[T],
        ) -> Result<(), compositor::SurfaceError> {
            if let iced::Renderer::Wgpu(renderer) = renderer {
                renderer.with_primitives(|backend, primitives| {
                    present(
                        self,
                        backend,
                        surface,
                        primitives,
                        viewport,
                        background_color,
                        overlay,
                    )
                })
            }
            else {
                panic!("Incorrect renderer type!")
            }
        }

        fn screenshot<T: AsRef<str>>(
            &mut self,
            renderer: &mut Self::Renderer,
            _surface: &mut Self::Surface,
            viewport: &Viewport,
            background_color: Color,
            overlay: &[T],
        ) -> Vec<u8> {
            [].into()
        }
    }
}