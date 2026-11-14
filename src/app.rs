use crate::state::State;
use std::sync::Arc;

use wgpu::Instance;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::Window,
};

pub(crate) struct App {
    state: Option<State>,
    instance: Instance,
}

impl App {
    pub fn new(instance: Instance) -> Self {
        Self {
            state: None,
            instance,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            match State::new(window, &self.instance) {
                Ok(state) => {
                    self.state = Some(state);
                }
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        if state.window.id() != window_id {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => match state.render() {
                Ok(()) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                state.resize(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
