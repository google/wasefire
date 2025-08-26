// Copyright 2025 Google LLC
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

/// Raw pointer for a shared reference with dynamic lifetime.
///
/// This is appropriate for `&T`.
pub struct SharedPtr<T>(pub *const T);

unsafe impl<T: Sync> Send for SharedPtr<T> {}
unsafe impl<T: Sync> Sync for SharedPtr<T> {}

/// Raw pointer for an exclusive reference with dynamic lifetime.
///
/// This is appropriate for `&mut T`.
pub struct ExclusivePtr<T>(pub *mut T);

unsafe impl<T: Send> Send for ExclusivePtr<T> {}
unsafe impl<T: Sync> Sync for ExclusivePtr<T> {}
