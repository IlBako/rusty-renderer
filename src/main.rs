use wgpu::Instance;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod display;
mod fps;
mod gpu;
mod state;
mod tracer;
mod utils;
use app::App;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut inst_desc = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
        event_loop.owned_display_handle(),
    ));
    inst_desc.backends = wgpu::Backends::DX12;
    let instance = Instance::new(inst_desc);

    let mut app = App::new(instance);
    event_loop.run_app(&mut app).expect("event loop error");
}
