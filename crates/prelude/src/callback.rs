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

macro_rules! define {
    ($an:ident $n:ident $(, $x:ident)*) => {
        define!(#[cfg(not(feature = "native"))] $n $(, $x)*);
        define!(#[cfg(feature = "native")] $an $(, $x)*);
    };
    (#[$m:meta] $n:ident $(, $x:ident)*) => {
        #[$m] #[no_mangle]
        extern "C" fn $n (
            ptr: extern "C" fn(*const u8 $(, usize ${ignore($x)})*),
            this: *const u8 $(, $x: usize)*
        ) {
            ptr(this $(, $x)*);
        }
    };
}
define!(applet_cb0 cb0);
define!(applet_cb1 cb1, x0);
define!(applet_cb2 cb2, x0, x1);
define!(applet_cb3 cb3, x0, x1, x2);
