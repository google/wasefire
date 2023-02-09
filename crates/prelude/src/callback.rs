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
    ($n:ident $(, $x:ident)*) => {
        #[no_mangle]
        pub extern "C" fn $n(
            ptr: extern "C" fn(*mut u8 $(, usize ${ignore(x)})*),
            this: *mut u8 $(, $x: usize)*
        ) {
            ptr(this $(, $x)*);
        }
    };
}
define!(cb0);
define!(cb1, x0);
define!(cb2, x0, x1);
define!(cb3, x0, x1, x2);
