use smithay::backend::input::{InputBackend, InputEvent};

use crate::khigy::KhigyState;

impl KhigyState {
    pub fn process_input_event<I: InputBackend>(&mut self, _event: InputEvent<I>) {}
}
