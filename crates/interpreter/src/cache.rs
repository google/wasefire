// Copyright 2022 Google LLC
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

#[cfg(feature = "cache")]
mod impl_ {
    use lru::LruCache;

    pub type Cache<K, V> = LruCache<K, V>;
}

#[cfg(not(feature = "cache"))]
mod impl_ {
    use core::fmt::Debug;
    use core::marker::PhantomData;

    pub struct Cache<K, V> {
        phantom: PhantomData<(K, V)>,
    }

    impl<K, V> Debug for Cache<K, V> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Cache").finish()
        }
    }

    impl<K, V> Cache<K, V> {
        pub fn unbounded() -> Self {
            Self { phantom: PhantomData }
        }

        pub fn get(&mut self, _k: &K) -> Option<&V> {
            None
        }

        pub fn put(&mut self, _k: K, _v: V) -> Option<V> {
            None
        }
    }
}

pub use impl_::*;
