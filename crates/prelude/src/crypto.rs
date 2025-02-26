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

//! Provides API for cryptography.

#[cfg(feature = "api-crypto-cbc")]
pub mod cbc;
#[cfg(feature = "api-crypto-ccm")]
pub mod ccm;
#[cfg(feature = "api-crypto-ec")]
pub mod ec;
#[cfg(feature = "api-crypto-gcm")]
pub mod gcm;
#[cfg(feature = "internal-api-crypto-hash")]
pub mod hash;
