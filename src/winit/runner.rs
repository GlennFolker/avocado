use crate::incl::*;

pub struct WinitRunner;
impl WinitRunner {
    pub fn init(app: &mut App) -> &mut App {
        let event_loop = winit::EventLoop::new();
        let window = match app.res_or(WindowConfig::default).create(&event_loop) {
            Ok(window) => window,
            Err(err) => panic!("Couldn't create window: {:?}", err),
        };

        app
            .init_res::<ClearColor>()
            .insert_res_ns(WinitWindow(window))
            .insert_res_ns(EventLoop(event_loop));

        let window = &**app.res_ns::<WinitWindow>().unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = future::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).or_else(|| instance
            .enumerate_adapters(wgpu::Backends::all())
            .filter(|adapter| !surface.get_supported_formats(&adapter).is_empty())
            .next()
        ).expect("Couldn't request a fitting video adapter");

        let (device, queue) = match future::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features:
                    wgpu::Features::TEXTURE_BINDING_ARRAY |
                    wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                limits: adapter.limits(),
                label: None,
            },
            None,
        )) {
            Ok((device, queue)) => (device, queue),
            Err(err) => panic!("Couldn't request render device: {}", err),
        };

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: if app.res::<WindowConfig>().unwrap().vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &config);
        app
            .insert_res(SurfaceConfig { surface, config, size, })
            .insert_res(Renderer { device, queue, })
    }

    pub fn run(mut app: App) -> ! {
        #[derive(Default)]
        struct State {
            exit: Arc<RwLock<Option<ExitReason>>>,
        }

        let mut state = Some(State::default());
        app.exit_handle(Arc::clone(&state.as_ref().unwrap().exit));

        let event_loop = app.remove_res_ns::<EventLoop>().unwrap().0;
        event_loop.run(move |event, _, control_flow| {
            let (world, schedule) = app.unzip_mut();
            let window = &*world.non_send_resource::<WinitWindow>();

            match event {
                winit::Event::NewEvents(cause) => {
                    if cause == winit::StartCause::Init {
                        world.send_event(ResumeEvent);
                    }
                },
                winit::Event::WindowEvent { window_id, event } => {
                    if window_id == window.id() {
                        match event {
                            winit::WindowEvent::Resized(size) => world.send_event(WindowResizedEvent(size)),
                            winit::WindowEvent::Moved(pos) => world.send_event(WindowMovedEvent(pos)),
                            winit::WindowEvent::CloseRequested => {
                                world.send_event(ExitEvent::graceful());
                                world.send_event(SuspendEvent);
                            },
                            winit::WindowEvent::KeyboardInput {
                                input: winit::KeyboardInput {
                                    state, virtual_keycode, ..
                                },
                                ..
                            } => if let Some(vkey) = virtual_keycode && let Ok(key) = vkey.try_into() {
                                world.send_event(KeyboardEvent {
                                    pressed: state == winit::ElementState::Pressed,
                                    key,
                                });
                            },
                            winit::WindowEvent::ModifiersChanged(state) => world.send_event(KeyModifierEvent {
                                alt: state.alt(),
                                ctrl: state.ctrl(),
                                logo: state.logo(),
                                shift: state.shift(),
                            }),
                            _ => {},
                        }
                    }
                },
                winit::Event::MainEventsCleared => {
                    schedule.run(world);

                    let exit = state.as_ref().unwrap().exit.read();
                    let exit_code = match &*exit {
                        Some(ExitReason::Graceful) => {
                            log::info!("App exited gracefully");
                            Some(0)
                        },
                        Some(ExitReason::Error(msg)) => {
                            log::error!("App crashed: {}", msg);
                            Some(1)
                        },
                        None => None,
                    };

                    if let Some(exit_code) = exit_code {
                        *control_flow = winit::ControlFlow::ExitWithCode(exit_code);
                    }
                },
                winit::Event::LoopDestroyed => {
                    let app = mem::replace(&mut app, App::empty());
                    drop(app);

                    state = None;
                },
                _ => {},
            }
        })
    }
}
