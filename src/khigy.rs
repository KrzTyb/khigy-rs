use smithay::{
    input::{Seat, SeatState},
    reexports::{
        calloop::{EventLoop, LoopSignal},
        wayland_server::{Display, DisplayHandle},
    },
    wayland::{data_device::DataDeviceState, shell::xdg::XdgShellState},
};
pub struct KhigyState {
    pub start_time: std::time::Instant,
    pub display_handle: DisplayHandle,
    pub loop_signal: LoopSignal,
    pub xdg_shell_state: XdgShellState,
    pub seat_state: SeatState<KhigyState>,
    pub data_device_state: DataDeviceState,
    pub seat: Seat<Self>,
}

impl KhigyState {
    pub fn new(event_loop: &mut EventLoop<crate::LoopData>, display: &mut Display<Self>) -> Self {
        let start_time = std::time::Instant::now();
        // Get the loop signal, used to stop the event loop
        let loop_signal = event_loop.get_signal();
        let display_handle = display.handle();

        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);

        let mut seat_state = SeatState::new();

        let data_device_state = DataDeviceState::new::<Self>(&display_handle);

        // A seat is a group of keyboards, pointer and touch devices.
        // A seat typically has a pointer and maintains a keyboard focus and a pointer focus.
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&display_handle, "winit");

        // Notify clients that we have a keyboard, for the sake of the example we assume that keyboard is always present.
        // You may want to track keyboard hot-plug in real compositor.
        seat.add_keyboard(Default::default(), 200, 25).unwrap();

        // Notify clients that we have a pointer (mouse)
        // Here we assume that there is always pointer plugged in
        seat.add_pointer();

        Self {
            start_time,
            display_handle,
            loop_signal,
            xdg_shell_state,
            seat_state,
            data_device_state,
            seat,
        }
    }
}
