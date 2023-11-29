// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Returns whether this statement was already executed.
#[macro_export]
macro_rules! executed {
    () => {{
        use $crate::{AtomicBool, Ordering};
        static __X: AtomicBool = AtomicBool::new(false);
        __X.swap(true, Ordering::Relaxed)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn executed_ok() {
        fn test() -> bool {
            executed!()
        }
        assert!(!test());
        assert!(test());
        assert!(test());
    }
}
