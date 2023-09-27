use smithay::reexports::{
    calloop::{EventLoop, LoopSignal},
    wayland_server::{Display, DisplayHandle},
};

use crate::backend;

pub mod handlers;

pub enum Backend {
    Winit(backend::winit::WinitBackend),
    Invalid,
}

pub struct State {
    pub display_handle: DisplayHandle,
    pub loop_signal: LoopSignal,
}

pub struct LoopData {
    pub state: State,
    pub display: Display<State>,
    pub backend: Backend,
}

impl State {
    pub fn new(event_loop: &mut EventLoop<LoopData>, display: &mut Display<Self>) -> Self {
        // Get the loop signal, used to stop the event loop
        let loop_signal = event_loop.get_signal();
        let display_handle = display.handle();

        Self {
            display_handle,
            loop_signal,
        }
    }
}
