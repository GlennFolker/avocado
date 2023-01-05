use avocado_core::prelude::*;
use avocado_input::prelude::*;
use avocado_utils::prelude::*;

use crate::{
    WinitEventLoop, WinitWindow,
    WindowConfig, SurfaceConfig, ClearColor, Renderer,
    WindowResizedEvent, WindowMovedEvent, SuspendEvent, ResumeEvent,
    KeyboardEvent, KeyModifierEvent,
    KeyCodeExt as _,
};
use winit::{
    event::{
        StartCause,
        Event, WindowEvent,
        KeyboardInput, ElementState,
    },
    event_loop::{
        EventLoop, ControlFlow,
    },
};
use std::{
    mem,
    sync::Arc,
};

pub struct WinitRunner;
impl WinitRunner {
    pub fn init(app: &mut App) -> &mut App {
        let event_loop = EventLoop::new();
        let window = match app.res_or(WindowConfig::default).create(&event_loop) {
            Ok(window) => window,
            Err(err) => panic!("Couldn't create window: {:?}", err),
        };

        app
            .init_res::<ClearColor>()
            .insert_res_ns(WinitWindow(window))
            .insert_res_ns(WinitEventLoop(event_loop));

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

        let event_loop = app.remove_res_ns::<WinitEventLoop>().unwrap().0;
        event_loop.run(move |event, _, control_flow| {
            let (world, schedule) = app.unzip_mut();
            let window = &*world.non_send_resource::<WinitWindow>();

            match event {
                Event::NewEvents(cause) => {
                    if cause == StartCause::Init {
                        world.send_event(ResumeEvent);
                    }
                },
                Event::WindowEvent { window_id, event } => {
                    if window_id == window.id() {
                        match event {
                            WindowEvent::Resized(size) => world.send_event(WindowResizedEvent(size)),
                            WindowEvent::Moved(pos) => world.send_event(WindowMovedEvent(pos)),
                            WindowEvent::CloseRequested => {
                                world.send_event(ExitEvent::graceful());
                                world.send_event(SuspendEvent);
                            },
                            WindowEvent::KeyboardInput {
                                input: KeyboardInput {
                                    state, virtual_keycode, ..
                                },
                                ..
                            } => if let Some(vkey) = virtual_keycode && let Some(key) = KeyCode::from_vkey(vkey) {
                                world.send_event(KeyboardEvent {
                                    pressed: state == ElementState::Pressed,
                                    key,
                                });
                            },
                            WindowEvent::ModifiersChanged(state) => world.send_event(KeyModifierEvent {
                                alt: state.alt(),
                                ctrl: state.ctrl(),
                                logo: state.logo(),
                                shift: state.shift(),
                            }),
                            _ => {},
                        }
                    }
                },
                Event::MainEventsCleared => {
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
                        *control_flow = ControlFlow::ExitWithCode(exit_code);
                    }
                },
                Event::LoopDestroyed => {
                    let app = mem::replace(&mut app, App::empty());
                    drop(app);

                    state = None;
                },
                _ => {},
            }
        })
    }
}
