// Temporary solution, remove later
#![allow(dead_code)]

mod backend;
mod khigy;

use anyhow::{Context, Result};
use smithay::{
    reexports::{calloop::EventLoop, wayland_server::Display},
    wayland::socket::ListeningSocketSource,
};

fn main() -> Result<()> {
    env_logger::init();

    let mut event_loop: EventLoop<khigy::LoopData> =
        EventLoop::try_new_high_precision().with_context(|| "Failed to create event loop")?;

    let mut display = build_display(&mut event_loop)?;
    let state = khigy::State::new(&mut event_loop, &mut display);

    let mut data = khigy::LoopData {
        state,
        display,
        backend: khigy::Backend::Invalid,
    };

    backend::create_backend(&mut event_loop, &mut data)?;

    event_loop.run(None, &mut data, move |_data| {
        // Event loop
    })?;

    // Drop content before logger
    drop(event_loop);
    drop(data);

    Ok(())
}

fn build_display(event_loop: &mut EventLoop<khigy::LoopData>) -> Result<Display<khigy::State>> {
    let display = Display::new()?;

    let socket_source =
        ListeningSocketSource::new_auto().with_context(|| "Failed to create wayland socket")?;

    log::info!(
        "Created Wayland socket: {}",
        socket_source.socket_name().to_str().unwrap_or("Invalid")
    );

    event_loop
        .handle()
        .insert_source(socket_source, |_stream, _, _data| {})
        .with_context(|| "Failed to start listening on Wayland socket")?;

    Ok(display)
}
