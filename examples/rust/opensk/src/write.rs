pub struct WasefireWrite;

impl Default for WasefireWrite {
    fn default() -> Self {
        Self {}
    }
}

impl core::fmt::Write for WasefireWrite {
    fn write_str(&mut self, _: &str) -> core::fmt::Result {
        todo!()
    }
}
