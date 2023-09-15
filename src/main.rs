mod backend;
mod handlers;
mod input;
mod khigy;

use anyhow::Result;
use smithay::reexports::{calloop::EventLoop, wayland_server::Display};

pub struct LoopData {
    state: khigy::KhigyState,
    display: Display<khigy::KhigyState>,
}

fn main() -> Result<()> {
    env_logger::init();

    let mut event_loop: EventLoop<LoopData> = EventLoop::try_new()?;

    let mut display: Display<khigy::KhigyState> = Display::new()?;
    let state = khigy::KhigyState::new(&mut event_loop, &mut display);

    let mut data = LoopData { state, display };

    if cfg!(feature = "winit") {
        #[cfg(feature = "winit")]
        backend::winit::run(&mut event_loop, &mut data)?;
    } else if cfg!(feature = "vulkan") {
        #[cfg(feature = "vulkan")]
        backend::vulkan::run(&mut event_loop, &mut data)?;
    } else {
        return Err(anyhow::anyhow!("Unsupported backend!"));
    }

    event_loop.run(None, &mut data, move |_data| {
        // Event loop
    })?;

    Ok(())
}
