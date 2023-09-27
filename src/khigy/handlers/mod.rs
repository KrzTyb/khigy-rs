use smithay::delegate_output;

mod input;

//
// Wl Output & Xdg Output
//
delegate_output!(super::State);
