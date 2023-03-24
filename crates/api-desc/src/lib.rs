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

use std::io::Write;

use clap::ValueEnum;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

mod button;
mod clock;
mod crypto;
mod debug;
mod id;
mod led;
mod macros;
mod rng;
mod scheduling;
mod store;
mod usb;

pub use id::{Id, Name};

#[derive(Debug, Clone)]
pub struct Api(Vec<Item>);

impl Default for Api {
    fn default() -> Self {
        Api(vec![
            button::new(),
            clock::new(),
            crypto::new(),
            debug::new(),
            led::new(),
            rng::new(),
            scheduling::new(),
            store::new(),
            usb::new(),
            item! {
                /// Board-specific syscalls.
                ///
                /// Those calls are forwarded by the scheduler.
                fn syscall "s" { x1: usize, x2: usize, x3: usize, x4: usize } -> { res: isize }
            },
        ])
    }
}

#[derive(Copy, Clone, ValueEnum)]
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
    Enum(Enum),
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
struct Variant {
    docs: Vec<String>,
    name: String,
}

#[derive(Debug, Clone)]
struct Fn {
    docs: Vec<String>,
    name: String,
    link: String,
    params: Vec<Field>,
    results: Vec<Field>,
}

#[derive(Debug, Clone)]
struct Field {
    docs: Vec<String>,
    name: String,
    type_: Type,
}

#[derive(Debug, Clone)]
enum Type {
    Integer {
        signed: bool,
    },
    Pointer {
        mutable: bool,

        /// Pointed type (None if opaque).
        type_: Option<Box<Type>>,
    },
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
            Item::Fn(x) => x.host(),
            Item::Mod(x) => x.host(),
        }
    }

    fn wasm_rust(&self) -> TokenStream {
        match self {
            Item::Enum(x) => x.wasm_rust(),
            Item::Fn(x) => x.wasm_rust(),
            Item::Mod(x) => x.wasm_rust(),
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        match self {
            Item::Enum(x) => x.wasm_assemblyscript(output, path),
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
            impl From<#name> for crate::U32<isize> {
                fn from(x: #name) -> Self { (!(x as u32)).into() }
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
            impl #name {
                pub fn to_result(x: isize) -> Result<usize, Self> {
                    let y = x as usize;
                    if x < 0 { Err((!y).into()) } else { Ok(y) }
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
                type Error = ();

                fn try_from(x: u32) -> Result<Self, Self::Error> {
                    if x < #num_variants as u32 {
                        Ok(unsafe { core::mem::transmute(x) })
                    } else {
                        Err(())
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

impl Fn {
    fn host(&self) -> TokenStream {
        let Fn { docs, name, link, params, results } = self;
        let name = format_ident!("{}", name);
        let doc = format!("Module of [`{name}`]({name}::Sig).");
        let params: Vec<_> = params.iter().map(|x| x.host()).collect();
        let results: Vec<_> = results.iter().map(|x| x.host()).collect();
        quote! {
            #[doc = #doc]
            pub mod #name {
                #(#[doc = #docs])*
                #[derive(Debug, Default, Copy, Clone)]
                #[repr(C)]
                pub struct Sig;

                /// Parameters of [Sig].
                #[derive(Debug, Default, Copy, Clone)]
                #[repr(C)]
                pub struct Params { #(#params,)* }

                /// Results of [Sig].
                #[derive(Debug, Default, Copy, Clone)]
                #[repr(C)]
                pub struct Results { #(#results,)* }

                #[sealed::sealed] impl crate::ArrayU32 for Params {}
                #[sealed::sealed] impl crate::ArrayU32 for Results {}
                #[sealed::sealed]
                impl crate::Signature for Sig {
                    const NAME: &'static str = #link;
                    type Params = Params;
                    type Results = Results;
                }
            }
        }
    }

    fn wasm_rust(&self) -> TokenStream {
        let Fn { docs, name, link, params, results } = self;
        let name = format_ident!("{}", name);
        let doc = format!("Module of [`{name}`]({name}()).");
        let params_doc = format!("Parameters of [`{name}`](super::{name}())");
        let results_doc = format!("Results of [`{name}`](super::{name}())");
        let params: Vec<_> = params.iter().map(|x| x.wasm_rust()).collect();
        let results: Vec<_> = results.iter().map(|x| x.wasm_rust()).collect();
        let fn_params = if params.is_empty() { None } else { Some(quote!(params: #name::Params)) };
        let fn_results = if results.is_empty() { None } else { Some(quote!(-> #name::Results)) };
        quote! {
            #[doc = #doc]
            pub mod #name {
                #[doc = #params_doc]
                #[derive(Debug, Copy, Clone)]
                #[repr(C)]
                pub struct Params { #(#params,)* }

                #[doc = #results_doc]
                #[derive(Debug, Copy, Clone)]
                #[repr(C)]
                pub struct Results { #(#results,)* }
            }
            #[cfg(not(feature = "test"))]
            extern "C" {
                #(#[doc = #docs])*
                #[link_name = #link]
                pub fn #name(#fn_params) #fn_results;
            }
            #[cfg(feature = "test")]
            pub unsafe fn #name(#fn_params) #fn_results {
                panic!("Applet API is not linked in unit tests.");
            }
        }
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Fn { docs, name, link, params, results } = self;
        write_docs(output, docs, path)?;
        writeln!(output, r#"{path:#}@external("env", "{link}")"#)?;
        writeln!(output, "{path:#}export declare function {path}{name}(")?;
        write_items(output, params, |output, param| param.wasm_assemblyscript(output, path))?;
        if let Some(result) = results.get(0) {
            write_docs(output, &result.docs, path)?;
        }
        write!(output, "{path:#}): ")?;
        match &results[..] {
            [] => write!(output, "void")?,
            [result] => result.type_.wasm_assemblyscript(output)?,
            _ => unimplemented!("multi-value is not supported in AssemblyScript"),
        }
        writeln!(output)
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
        let Field { docs, name, type_ } = self;
        let name = format_ident!("{}", name);
        let type_ = type_.wasm_rust();
        quote!(#(#[doc = #docs])* pub #name: crate::U32<#type_>)
    }

    fn wasm_rust(&self) -> TokenStream {
        let Field { docs, name, type_ } = self;
        let name = format_ident!("{}", name);
        let type_ = type_.wasm_rust();
        quote!(#(#[doc = #docs])* pub #name: #type_)
    }

    fn param(&self) -> TokenStream {
        let Field { docs: _, name, type_ } = self;
        let name = format_ident!("{}", name);
        let type_ = type_.wasm_rust();
        quote!(#name: #type_)
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Field { docs, name, type_ } = self;
        let path = Path::Mod { name: "", prev: path };
        write_docs(output, docs, &path)?;
        write!(output, "{path:#}{name}: ")?;
        type_.wasm_assemblyscript(output)?;
        writeln!(output, ",")
    }
}

impl Type {
    fn wasm_rust(&self) -> TokenStream {
        match self {
            Type::Integer { signed: true } => quote!(isize),
            Type::Integer { signed: false } => quote!(usize),
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
            Type::Integer { signed: true } => write!(output, "isize"),
            Type::Integer { signed: false } => write!(output, "usize"),
            // TODO: Is there a way to decorate this better?
            Type::Pointer { mutable: _, type_: _ } => write!(output, "usize"),
            // TODO: Is there a way to decorate this better?
            Type::Function { params: _ } => write!(output, "usize"),
        }
    }
}

impl Variant {
    fn wasm_rust(&self) -> TokenStream {
        let Variant { docs, name } = self;
        let name = format_ident!("{}", name);
        quote!(#(#[doc = #docs])* #name)
    }

    fn wasm_assemblyscript(&self, output: &mut dyn Write, path: &Path) -> std::io::Result<()> {
        let Variant { docs, name } = self;
        let path = Path::Mod { name: "", prev: path };
        write_docs(output, docs, &path)?;
        writeln!(output, "{path:#}{name},")
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

impl<'a> std::fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, prev) = match self {
            Path::Empty => return Ok(()),
            Path::Mod { name, prev } => (name, prev),
        };
        if f.alternate() {
            write!(f, "{prev:#}  ")
        } else {
            write!(f, "{prev}{name}_")
        }
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
