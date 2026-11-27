use crate::state::State;
use std::{collections::HashSet, sync::Arc};

use wgpu::Instance;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::Window,
};

pub struct App {
    state: Option<State>,
    instance: Instance,
    input: InputState,
}

#[derive(Default)]
pub struct InputState {
    middle_mouse_pressed: bool,
    pressed_keys: HashSet<KeyCode>,
    modifiers: ModifiersState,
}

impl App {
    #[must_use]
    pub fn new(instance: Instance) -> Self {
        Self {
            state: None,
            instance,
            input: InputState::default(),
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
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.input.modifiers = new_modifiers.state();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: key_state,
                        repeat,
                        ..
                    },
                ..
            } => match key_state {
                ElementState::Pressed => {
                    self.input.pressed_keys.insert(key_code);

                    if !repeat {
                        match key_code {
                            KeyCode::Space => state.reset_camera(),
                            KeyCode::Enter => state.add_random_sphere(),
                            KeyCode::Backspace => state.remove_last_sphere(),
                            _ => {}
                        }
                    }
                }
                ElementState::Released => {
                    self.input.pressed_keys.remove(&key_code);
                }
            },
            WindowEvent::Focused(is_focused) => {
                if !is_focused {
                    self.input = InputState::default();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Middle {
                    self.input.middle_mouse_pressed = state.is_pressed();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let notches = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 30.0,
                };

                state.dolly(notches);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event
            && self.input.middle_mouse_pressed
        {
            if self.input.modifiers.shift_key() {
                state.pan(dx, dy);
            } else {
                state.orbit(dx, dy);
            }
        }
    }
}
