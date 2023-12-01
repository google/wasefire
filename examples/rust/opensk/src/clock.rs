use opensk_lib::api::clock::Clock;

pub struct WasefireClock {}

impl Default for WasefireClock {
    fn default() -> Self {
        Self {}
    }
}

impl Clock for WasefireClock {
    type Timer = Self;

    fn make_timer(&mut self, milliseconds: usize) -> Self::Timer {
        todo!()
    }

    fn is_elapsed(&mut self, timer: &Self::Timer) -> bool {
        todo!()
    }
}
