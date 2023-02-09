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

use proc_macro::TokenStream;

#[proc_macro]
pub fn wasm(input: TokenStream) -> TokenStream {
    assert!(input.is_empty());
    api_desc::Api::default().wasm_rust().into()
}

#[proc_macro]
pub fn host(input: TokenStream) -> TokenStream {
    assert!(input.is_empty());
    api_desc::Api::default().host().into()
}
