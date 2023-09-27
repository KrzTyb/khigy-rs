pub mod winit;

use crate::khigy::LoopData;
use anyhow::Result;
use smithay::reexports::calloop::EventLoop;

pub fn create_backend(event_loop: &mut EventLoop<LoopData>, data: &mut LoopData) -> Result<()> {
    winit::create_backend(event_loop, data)?;

    Ok(())
}
