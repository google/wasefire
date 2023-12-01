use opensk_lib::api::user_presence::UserPresence;

use crate::WasefireEnv;

impl UserPresence for WasefireEnv {
    fn check_init(&mut self) {
        todo!()
    }

    fn wait_with_timeout(
        &mut self, timeout_ms: usize,
    ) -> opensk_lib::api::user_presence::UserPresenceResult {
        todo!()
    }

    fn check_complete(&mut self) {
        todo!()
    }
}
