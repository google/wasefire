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

use std::ffi::CString;
use std::io::Write;
use std::ops;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

#[cfg(feature = "api-button")]
mod button;
#[cfg(feature = "api-clock")]
mod clock;
#[cfg(feature = "internal-api-crypto")]
mod crypto;
mod debug;
#[cfg(feature = "internal-api-fingerprint")]
mod fingerprint;
#[cfg(feature = "api-gpio")]
mod gpio;
mod id;
#[cfg(feature = "api-led")]
mod led;
mod macros;
#[cfg(feature = "internal-api-platform")]
mod platform;
#[cfg(feature = "internal-api-radio")]
mod radio;
#[cfg(feature = "api-rng")]
mod rng;
mod scheduling;
#[cfg(feature = "internal-api-store")]
mod store;
#[cfg(feature = "api-timer")]
mod timer;
#[cfg(feature = "api-uart")]
mod uart;
#[cfg(feature = "internal-api-usb")]
mod usb;

pub use id::{Id, Name};

#[derive(Debug, Clone)]
pub struct Api(Vec<Item>);

impl Default for Api {
    fn default() -> Self {
        Api(vec![
            #[cfg(feature = "api-button")]
            button::new(),
            #[cfg(feature = "api-clock")]
            clock::new(),
            #[cfg(feature = "internal-api-crypto")]
            crypto::new(),
            debug::new(),
            #[cfg(feature = "internal-api-fingerprint")]
            fingerprint::new(),
            #[cfg(feature = "api-gpio")]
            gpio::new(),
            #[cfg(feature = "api-led")]
            led::new(),
            #[cfg(feature = "internal-api-platform")]
            platform::new(),
            #[cfg(feature = "internal-api-radio")]
            radio::new(),
            #[cfg(feature = "api-rng")]
            rng::new(),
            scheduling::new(),
            #[cfg(feature = "internal-api-store")]
            store::new(),
            #[cfg(feature = "api-timer")]
            timer::new(),
            #[cfg(feature = "api-uart")]
            uart::new(),
            #[cfg(feature = "internal-api-usb")]
            usb::new(),
            item! {
                /// Board-specific syscalls.
                ///
                /// Those calls are directly forwarded to the board by the scheduler.
                fn syscall "s" { x1: usize, x2: usize, x3: usize, x4: usize } -> usize
            },
        ])
    }
}

#[derive(Copy, Clone)]
pub enum Lang {
    C,
    Assemblyscript,
}

impl Api {
    pub fn host(&self) -> TokenStream {
        Mod::body(None, &self.0)
    }

    pub fn wasm(&self, output: &mut dyn Write, lang: Lang) -> std::io::Result<()> {
        match lang {
            Lang::C => unimplemented!(),
            Lang::Assemblyscript => self.wasm_assemblyscript(output),
        }
    }

    pub fn wasm_markdown(&self) -> &'static str {
        include_str!("api.md")
    }

    pub fn wasm_rust(&self) -> TokenStream {
        let items: Vec<_> = self.0.iter().map(|x| x.wasm_rust()).collect();
        quote!(#(#items)*)
    }

    pub fn wasm_assemblyscript(&self, output: &mut dyn Write) -> std::io::Result<()> {
        write_items(output, &self.0, |output, item| item.wasm_assemblyscript(output, &Path::Empty))
    }
}

#[derive(Debug, Clone)]
enum Item {
    #[cfg_attr(not(feature = "api-button"), allow(dead_code))]
    Enum(Enum),
    Struct(Struct),
    Fn(Fn),
    Mod(Mod),
}

#[derive(Debug, Clone)]
struct Enum {
    docs: Vec<String>,
    name: String,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
struct Struct {
    docs: Vec<String>,
    name: String,
    fields: Vec<Field>,
}

#[derive(Debug, Clone)]
struct Variant {
    docs: Vec<String>,
    name: String,
    value: u32,
}

#[derive(Debug, Clone)]
struct Fn {
    docs: Vec<String>,
    name: String,
    link: String,
    params: Vec<Field>,
    result: Type,
}

#[derive(Debug, Clone)]
struct Field {
    docs: Vec<String>,
    name: String,
    type_: Type,
}

#[derive(Debug, Clone)]
enum Type {
    Never,
    Unit,
    #[cfg_attr(not(feature = "api-store"), allow(dead_code))]
    Bool,
    Integer {
        signed: bool,
        bits: Option<usize>,
    },
    #[cfg_attr(not(feature = "api-radio-ble"), allow(dead_code))]
    Array {
        type_: Box<Type>,
        length: usize,
    },
    Pointer {
        mutable: bool,

        /// Pointed type (None if opaque).
        type_: Option<Box<Type>>,
    },
    #[cfg_attr(not(feature = "api-button"), allow(dead_code))]
    Function {
        params: Vec<Field>,
    },
}

#[derive(Debug, Clone)]
struct Mod {
    docs: Vec<String>,
    name: String,
    items: Vec<Item>,
}

impl Item {
    fn host(&self) -> TokenStream {
        match self {
            Item::Enum(x) => x.host(),
            Item::Struct(x) => x.host(),
            Item::Fn(x) => x.host(),
            Item::Mod(x) => x.host(),
        }
    }

    fn wasm_rust(&self) -> TokenStream {
        match self {
            Item::Enum(x) => x.wasm_rust(),
            Item::Struct(x) => x.wasm_rust(),
            Item::Fn(x) => x.wasm_rust(),
            Item::Mod(x) => x.wasm_rust(),
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        match self {
            Item::Enum(x) => x.wasm_assemblyscript(output, path),
            Item::Struct(x) => x.wasm_assemblyscript(output, path),
            Item::Fn(x) => x.wasm_assemblyscript(output, path),
            Item::Mod(x) => x.wasm_assemblyscript(output, path),
        }
    }
}

impl Enum {
    fn host(&self) -> TokenStream {
        let definition = self.definition();
        let name = format_ident!("{}", self.name);
        quote! {
            #definition
            impl From<#name> for crate::U32<usize> {
                fn from(x: #name) -> Self { (x as u32).into() }
            }
        }
    }

    fn wasm_rust(&self) -> TokenStream {
        let definition = self.definition();
        let name = format_ident!("{}", self.name);
        quote! {
            #definition
            impl From<usize> for #name {
                fn from(x: usize) -> Self {
                    (x as u32).try_into().unwrap()
                }
            }
        }
    }

    fn definition(&self) -> TokenStream {
        let Enum { docs, name, variants } = self;
        let name = format_ident!("{}", name);
        let variants: Vec<_> = variants.iter().map(|x| x.wasm_rust()).collect();
        let num_variants = variants.len();
        quote! {
            #(#[doc = #docs])*
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
            #[repr(u32)]
            pub enum #name { #(#variants),* }
            impl TryFrom<u32> for #name {
                type Error = wasefire_error::Error;

                fn try_from(x: u32) -> Result<Self, Self::Error> {
                    if x < #num_variants as u32 {
                        // SAFETY: See `tests::enum_values_are_valid()`.
                        Ok(unsafe { core::mem::transmute(x) })
                    } else {
                        Err(wasefire_error::Error::user(wasefire_error::Code::OutOfBounds))
                    }
                }
            }
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Enum { docs, name, variants } = self;
        write_docs(output, docs, path)?;
        writeln!(output, "{path:#}enum {path}{name} {{")?;
        write_items(output, variants, |output, variant| variant.wasm_assemblyscript(output, path))?;
        writeln!(output, "{path:#}}}")
    }
}

impl Struct {
    fn host(&self) -> TokenStream {
        self.definition(|x| x.host())
    }

    fn wasm_rust(&self) -> TokenStream {
        self.definition(|x| x.wasm_rust())
    }

    fn definition(&self, field: impl ops::Fn(&Field) -> TokenStream) -> TokenStream {
        let Struct { docs, name, fields } = self;
        let name = format_ident!("{}", name);
        let fields: Vec<_> = fields.iter().map(field).collect();
        quote! {
            #(#[doc = #docs])*
            #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
            #[repr(C)]
            pub struct #name { #(#fields),* }
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Struct { docs, name, fields } = self;
        write_docs(output, docs, path)?;
        writeln!(output, "{path:#}class {path}{name} {{")?;
        write_items(output, fields, |output, field| field.wasm_assemblyscript(output, path, ";"))?;
        writeln!(output, "{path:#}}}")
    }
}

impl Fn {
    fn host(&self) -> TokenStream {
        let Fn { docs, name, link, params, result } = self;
        let name = format_ident!("{}", name);
        let doc = format!("Module of [`{name}`]({name}::Sig).");
        let params: Vec<_> = params.iter().map(|x| x.host()).collect();
        let result = result.host();
        quote! {
            #[doc = #doc]
            pub mod #name {
                #(#[doc = #docs])*
                #[derive(Debug, Default, Copy, Clone)]
                #[repr(C)]
                pub struct Sig;

                /// Parameters of [Sig].
                #[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
                #[repr(C)]
                pub struct Params { #(#params,)* }

                #[sealed::sealed] impl crate::ArrayU32 for Params {}
                #[sealed::sealed]
                impl crate::Signature for Sig {
                    const NAME: &'static str = #link;
                    type Params = Params;
                    type Result = #result;
                }
            }
        }
    }

    fn wasm_rust(&self) -> TokenStream {
        let Fn { docs, name, link, params, result: _ } = self;
        let name = format_ident!("{name}");
        let name_wasm = format_ident!("{name}_wasm");
        let env_link = format!("env_{link}");
        let ffi_link = syn::LitCStr::new(&CString::new(link.clone()).unwrap(), Span::call_site());
        let doc = format!("Module of [`{name}`]({name}()).");
        let params_doc = format!("Parameters of [`{name}`](super::{name}())");
        let params: Vec<_> = params.iter().map(|x| x.wasm_rust()).collect();
        let names_wasm: Vec<_> = self.params.iter().map(|x| format_ident!("{}", x.name)).collect();
        let params_wasm: Vec<_> = self.params.iter().map(|x| x.param()).collect();
        let (fn_params, let_params, let_params_wasm) = if params.is_empty() {
            (None, quote!(let ffi_params = core::ptr::null();), None)
        } else {
            (
                Some(quote!(params: #name::Params)),
                quote!(let ffi_params = &params as *const _ as *const u32;),
                Some(quote!(let #name::Params { #(#names_wasm,)* } = params;)),
            )
        };
        quote! {
            #[doc = #doc]
            pub mod #name {
                #[doc = #params_doc]
                #[derive(Debug, Copy, Clone)]
                #[repr(C)]
                pub struct Params { #(#params,)* }
            }
            #[cfg(not(feature = "native"))]
            unsafe extern "C" {
                #[link_name = #link]
                unsafe fn #name_wasm(#(#params_wasm,)*) -> isize;
            }
            #[cfg(not(feature = "native"))]
            #(#[doc = #docs])*
            pub unsafe fn #name(#fn_params) -> isize {
                #let_params_wasm
                unsafe { #name_wasm(#(#names_wasm,)*) }
            }
            #[cfg(feature = "native")]
            #[unsafe(export_name = #env_link)]
            #[linkage = "weak"]
            pub unsafe extern "C" fn #name(#fn_params) -> isize {
                #let_params
                crate::wasm::native::env_dispatch(#ffi_link.as_ptr(), ffi_params)
            }
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Fn { docs, name, link, params, result: _ } = self;
        write_docs(output, docs, path)?;
        writeln!(output, r#"{path:#}@external("env", "{link}")"#)?;
        writeln!(output, "{path:#}export declare function {path}{name}(")?;
        write_items(output, params, |output, param| param.wasm_assemblyscript(output, path, ","))?;
        writeln!(output, "{path:#}): i32")
    }
}

impl Mod {
    fn host(&self) -> TokenStream {
        let Mod { docs, name, items } = self;
        let ident = format_ident!("{}", name);
        let body = Mod::body(Some(name), items);
        quote! {
            #(#[doc = #docs])*
            pub mod #ident {
                #body
            }
        }
    }

    fn wasm_rust(&self) -> TokenStream {
        let Mod { docs, name, items } = self;
        let name = format_ident!("{}", name);
        let items: Vec<_> = items.iter().map(|x| x.wasm_rust()).collect();
        quote! {
            #(#[doc = #docs])*
            pub mod #name {
                #(#items)*
            }
        }
    }

    fn body(name: Option<&str>, items: &[Item]) -> TokenStream {
        let mut api = Vec::new();
        let mut ident_camels = Vec::new();
        let mut inner = Vec::new();
        let mut merge = Vec::new();
        let mut descriptor = Vec::new();
        let mut iter = Vec::new();
        let mut id = Vec::new();
        let mut erase = Vec::new();
        for item in items {
            match item {
                Item::Enum(_) => (),
                Item::Struct(_) => (),
                Item::Fn(Fn { name, .. }) => {
                    let doc = format!("Selector for [`{name}`]({name}::Sig).");
                    let name_camel = camel(name);
                    let name = format_ident!("{}", name);
                    let pat = quote!(Api::#name_camel);
                    api.push(quote!(#[doc = #doc] #name_camel(T::Merged<#name::Sig>),));
                    ident_camels.push(name_camel.clone());
                    inner.push(quote!(T::Merged<#name::Sig>));
                    merge.push(quote!(#pat(_) => Api::#name_camel(T::merge(erased)),));
                    descriptor
                        .push(quote!(#pat(_) => <#name::Sig as crate::Signature>::descriptor(),));
                    iter.push(quote!(output.push(wrap(Api::#name_camel(#name::Sig)));));
                    id.push(quote!(#pat(_) => Api::#name_camel(#name::Sig),));
                    erase.push(quote!(#pat(x) => <T as crate::Dispatch>::erase(x),));
                }
                Item::Mod(Mod { name, .. }) => {
                    let doc = format!("Selector for [`{name}`]({name}).");
                    let name_camel = camel(name);
                    let name = format_ident!("{}", name);
                    let pat = quote!(Api::#name_camel(x) =>);
                    api.push(quote!(#[doc = #doc] #name_camel(#name::Api<T>),));
                    ident_camels.push(name_camel.clone());
                    inner.push(quote!(#name::Api<T>));
                    merge.push(quote!(#pat Api::#name_camel(x.merge(erased)),));
                    descriptor.push(quote!(#pat x.descriptor(),));
                    iter.push(quote! {
                        #name::Api::<crate::Id>::iter(output, |x| wrap(Api::#name_camel(x)));
                    });
                    id.push(quote!(#pat Api::#name_camel(x.id()),));
                    erase.push(quote!(#pat x.erase(),));
                }
            }
        }
        let items: Vec<_> = items.iter().map(|x| x.host()).collect();
        let doc = match name {
            Some(x) => format!(" API for [`{x}`](self)"),
            None => " Applet API".to_string(),
        };
        quote! {
            #(#items)*
            #[doc = #doc]
            #[derive(Debug)]
            pub enum Api<T: crate::Dispatch> { #(#api)* }
            impl<T: crate::Dispatch> Copy for Api<T> where #(#inner: Copy,)* {}
            impl<T: crate::Dispatch> Clone for Api<T> where #(#inner: Clone,)* {
                fn clone(&self) -> Self {
                    match self { #(Api::#ident_camels(x) => Api::#ident_camels(x.clone()),)* }
                }
            }
            impl Api<crate::Id> {
                pub fn merge<T: crate::Dispatch>(&self, erased: T::Erased) -> Api<T> {
                    match self { #(#merge)* }
                }
                pub fn descriptor(&self) -> crate::Descriptor {
                    match self { #(#descriptor)* }
                }
                // TODO: Find a solution to have this computed at compile time.
                pub fn iter<T>(output: &mut alloc::vec::Vec<T>, wrap: impl Fn(Self) -> T) {
                    #(#iter)*
                }
            }
            impl<T: crate::Dispatch> Api<T> {
                pub fn id(&self) -> Api<crate::Id> {
                    match self { #(#id)* }
                }
                pub fn erase(self) -> T::Erased {
                    match self { #(#erase)* }
                }
            }
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Mod { docs, name, items } = self;
        writeln!(output, "{path:#}// START OF MODULE {path}{name}")?;
        write_docs(output, docs, path)?;
        let inner_path = Path::Mod { name, prev: path };
        write_items(output, items, |output, item| item.wasm_assemblyscript(output, &inner_path))?;
        writeln!(output, "{path:#}// END OF MODULE {path}{name}")
    }
}

impl Field {
    fn host(&self) -> TokenStream {
        self.definition(|x| x.host())
    }

    fn wasm_rust(&self) -> TokenStream {
        self.definition(|x| x.wasm_rust())
    }

    fn definition(&self, quote_type: impl ops::Fn(&Type) -> TokenStream) -> TokenStream {
        let Field { docs, name, type_ } = self;
        let name = format_ident!("{}", name);
        let type_ = quote_type(type_);
        quote!(#(#[doc = #docs])* pub #name: #type_)
    }

    fn param(&self) -> TokenStream {
        let Field { docs: _, name, type_ } = self;
        let name = format_ident!("{}", name);
        let type_ = type_.wasm_rust();
        quote!(#name: #type_)
    }

    fn wasm_assemblyscript(
        &self, output: &mut dyn Write, path: &Path, separator: &str,
    ) -> std::io::Result<()> {
        let Field { docs, name, type_ } = self;
        let name = match name.as_str() {
            "private" => "private_",
            "public" => "public_",
            x => x,
        };
        let path = Path::Mod { name: "", prev: path };
        write_docs(output, docs, &path)?;
        write!(output, "{path:#}{name}: ")?;
        type_.wasm_assemblyscript(output)?;
        writeln!(output, "{separator}")
    }
}

impl Type {
    #[cfg(test)]
    fn is_param(&self) -> bool {
        match self {
            Type::Never | Type::Unit | Type::Bool => false,
            Type::Integer { bits: None, .. } => true,
            Type::Integer { bits: Some(32), .. } => true,
            Type::Integer { bits: Some(_), .. } => false,
            Type::Array { .. } => false,
            Type::Pointer { .. } => true,
            Type::Function { .. } => true,
        }
    }

    #[cfg(test)]
    fn is_result(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Type::Never | Type::Unit | Type::Bool => true,
            Type::Integer { signed: false, bits: None } => true,
            _ => false,
        }
    }

    /// Returns whether the type width is the pointer width.
    fn needs_u32(&self) -> bool {
        match self {
            Type::Never | Type::Unit | Type::Bool => false,
            Type::Integer { bits: None, .. } => true,
            Type::Integer { bits: Some(_), .. } => false,
            Type::Array { .. } => false,
            Type::Pointer { .. } => true,
            Type::Function { .. } => true,
        }
    }

    fn host(&self) -> TokenStream {
        let mut type_ = self.wasm_rust();
        if self.needs_u32() {
            type_ = quote!(crate::U32<#type_>);
        }
        type_
    }

    fn wasm_rust(&self) -> TokenStream {
        match self {
            Type::Never => quote!(!),
            Type::Unit => quote!(()),
            Type::Bool => quote!(bool),
            Type::Integer { signed: true, bits: None } => quote!(isize),
            Type::Integer { signed: false, bits: None } => quote!(usize),
            Type::Integer { signed: true, bits: Some(8) } => quote!(i8),
            Type::Integer { signed: false, bits: Some(8) } => quote!(u8),
            Type::Integer { signed: true, bits: Some(16) } => quote!(i16),
            Type::Integer { signed: false, bits: Some(16) } => quote!(u16),
            Type::Integer { signed: true, bits: Some(32) } => quote!(i32),
            Type::Integer { signed: false, bits: Some(32) } => quote!(u32),
            Type::Integer { signed: true, bits: Some(64) } => quote!(i64),
            Type::Integer { signed: false, bits: Some(64) } => quote!(u64),
            Type::Integer { .. } => unimplemented!(),
            Type::Array { type_, length } => {
                let type_ = type_.wasm_rust();
                quote!([#type_; #length])
            }
            Type::Pointer { mutable, type_ } => {
                let mutable = if *mutable { quote!(mut) } else { quote!(const) };
                let type_ = match type_ {
                    None => quote!(u8),
                    Some(x) => x.wasm_rust(),
                };
                quote!(*#mutable #type_)
            }
            Type::Function { params } => {
                let params: Vec<_> = params.iter().map(|x| x.param()).collect();
                quote!(extern "C" fn(#(#params),*))
            }
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write) -> std::io::Result<()> {
        match self {
            Type::Never | Type::Unit | Type::Bool => unreachable!(),
            Type::Integer { signed: true, bits: None } => write!(output, "isize"),
            Type::Integer { signed: false, bits: None } => write!(output, "usize"),
            Type::Integer { signed: true, bits: Some(8) } => write!(output, "i8"),
            Type::Integer { signed: false, bits: Some(8) } => write!(output, "u8"),
            Type::Integer { signed: true, bits: Some(16) } => write!(output, "i16"),
            Type::Integer { signed: false, bits: Some(16) } => write!(output, "u16"),
            Type::Integer { signed: true, bits: Some(32) } => write!(output, "i32"),
            Type::Integer { signed: false, bits: Some(32) } => write!(output, "u32"),
            Type::Integer { signed: true, bits: Some(64) } => write!(output, "i64"),
            Type::Integer { signed: false, bits: Some(64) } => write!(output, "u64"),
            Type::Integer { .. } => unimplemented!(),
            Type::Array { .. } => write!(output, "unimplemented"),
            // TODO: Is there a way to decorate this better?
            Type::Pointer { mutable: _, type_: _ } => write!(output, "usize"),
            // TODO: Is there a way to decorate this better?
            Type::Function { params: _ } => write!(output, "usize"),
        }
    }
}

impl Variant {
    fn wasm_rust(&self) -> TokenStream {
        let Variant { docs, name, value } = self;
        let name = format_ident!("{}", name);
        quote!(#(#[doc = #docs])* #name = #value)
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Variant { docs, name, value } = self;
        let path = Path::Mod { name: "", prev: path };
        write_docs(output, docs, &path)?;
        writeln!(output, "{path:#}{name} = {value},")
    }
}

fn camel(input: &str) -> Ident {
    let mut output = String::new();
    let mut begin = true;
    for c in input.chars() {
        if c == '_' {
            begin = true;
            continue;
        }
        if begin {
            begin = false;
            output.extend(c.to_uppercase());
        } else {
            output.push(c);
        }
    }
    Ident::new(&output, Span::call_site())
}

enum Path<'a> {
    Empty,
    Mod { name: &'a str, prev: &'a Path<'a> },
}

impl std::fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, prev) = match self {
            Path::Empty => return Ok(()),
            Path::Mod { name, prev } => (name, prev),
        };
        if f.alternate() { write!(f, "{prev:#}  ") } else { write!(f, "{prev}{name}_") }
    }
}

fn write_docs(output: &mut dyn Write, docs: &[String], path: &Path) -> std::io::Result<()> {
    for doc in docs {
        writeln!(output, "{path:#}//{doc}")?;
    }
    Ok(())
}

fn write_items<T>(
    output: &mut dyn Write, items: &[T],
    mut write: impl FnMut(&mut dyn Write, &T) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut first = true;
    for item in items {
        if !std::mem::replace(&mut first, false) {
            writeln!(output)?;
        }
        write(output, item)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    #[test]
    fn link_names_are_unique() {
        let mut seen = HashSet::new();
        let mut todo = Api::default().0;
        while let Some(item) = todo.pop() {
            match item {
                Item::Enum(_) => (),
                Item::Struct(_) => (),
                Item::Fn(Fn { link, .. }) => assert_eq!(seen.replace(link), None),
                Item::Mod(Mod { items, .. }) => todo.extend(items),
            }
        }
    }

    /// Makes sure enum values form a range starting at zero.
    ///
    /// This invariant is assumed by unsafe code.
    #[test]
    fn enum_values_are_valid() {
        let mut todo = Api::default().0;
        while let Some(item) = todo.pop() {
            match item {
                Item::Enum(Enum { variants, .. }) => {
                    let mut seen = HashMap::new();
                    for Variant { name, value, .. } in variants {
                        if let Some(other) = seen.insert(value, name.clone()) {
                            panic!("duplicate enum value {value} between {name} and {other}");
                        }
                    }
                    for value in 0 .. seen.len() as u32 {
                        assert!(seen.contains_key(&value), "skipped enum value {value}");
                    }
                }
                Item::Struct(_) => (),
                Item::Fn(_) => (),
                Item::Mod(Mod { items, .. }) => todo.extend(items),
            }
        }
    }

    #[test]
    fn params_are_u32() {
        fn test(link: &str, field: &Field) {
            let name = &field.name;
            assert!(field.type_.is_param(), "Param {name} of {link:?} is not U32");
        }
        let mut todo = Api::default().0;
        while let Some(item) = todo.pop() {
            match item {
                Item::Enum(_) => (),
                Item::Struct(_) => (),
                Item::Fn(Fn { link, params, .. }) => params.iter().for_each(|x| test(&link, x)),
                Item::Mod(Mod { items, .. }) => todo.extend(items),
            }
        }
    }

    #[test]
    fn results_are_valid() {
        let mut todo = Api::default().0;
        while let Some(item) = todo.pop() {
            match item {
                Item::Enum(_) => (),
                Item::Struct(_) => (),
                Item::Fn(Fn { link, result, .. }) => {
                    assert!(result.is_result(), "Result of {link:?} is invalid");
                }
                Item::Mod(Mod { items, .. }) => todo.extend(items),
            }
        }
    }
}
