// Copyright 2024 Google LLC
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

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;
use syn::spanned::Spanned;

#[cfg(test)]
mod tests;

// TODO: Use {parse_}quote_spanned.

#[proc_macro_derive(Wire, attributes(wire))]
pub fn derive_wire(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    print_item(derive_item(&item)).into()
}

#[proc_macro_attribute]
pub fn internal_wire(
    attr: proc_macro::TokenStream, item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    if !attr.is_empty() {
        return syn::Error::new(item.span(), "unexpected attribute").into_compile_error().into();
    }
    print_item(derive_item(&item)).into()
}

fn print_item(item: syn::Result<syn::ItemImpl>) -> TokenStream {
    match item {
        #[cfg(feature = "_dev")]
        Ok(_) => TokenStream::new(),
        #[cfg(not(feature = "_dev"))]
        Ok(x) => quote!(#x),
        Err(e) => e.into_compile_error(),
    }
}

fn derive_item(item: &syn::DeriveInput) -> syn::Result<syn::ItemImpl> {
    let mut attrs = Attrs::parse(&item.attrs, AttrsKind::Item)?;
    let wire = match attrs.crate_.take() {
        Attr::Absent => syn::Ident::new("wasefire_wire", Span::call_site()).into(),
        Attr::Present(_, x) => x,
    };
    let param = match attrs.param.take() {
        Attr::Absent => {
            let first = item.generics.lifetimes().next();
            first.map_or_else(
                || syn::Lifetime::new("'wire", Span::call_site()),
                |x| x.lifetime.clone(),
            )
        }
        Attr::Present(_, x) => x,
    };
    let type_param = syn::Lifetime::new(&format!("'_{}", param.ident), param.span());
    let (_, parameters, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    if generics.lifetimes().all(|x| x.lifetime != param) {
        generics.params.insert(0, parse_quote!(#param));
    }
    if let Attr::Present(_, where_) = attrs.where_.take() {
        generics.make_where_clause().predicates.extend(where_);
    }
    let (generics, _, where_clause) = generics.split_for_impl();
    let (schema, encode, decode) = match &item.data {
        syn::Data::Struct(x) => derive_struct(&mut attrs, &wire, &param, &item.ident, x)?,
        syn::Data::Enum(x) => derive_enum(&mut attrs, &wire, &item.ident, x)?,
        syn::Data::Union(_) => {
            return Err(syn::Error::new(item.span(), "unions are not supported"));
        }
    };
    if let Attr::Present(span, _) = attrs.range.take() {
        return Err(syn::Error::new(span, "range is only supported for enums"));
    }
    if let Attr::Present(span, _) = attrs.refine.take() {
        return Err(syn::Error::new(span, "refine is only supported for structs with one field"));
    }
    let ident = &item.ident;
    let statics: Vec<syn::Type> = match attrs.static_.take() {
        Attr::Absent => Vec::new(),
        Attr::Present(_, xs) => xs.into_iter().map(|x| parse_quote!(#x)).collect(),
    };
    let mut visitor = MakeType { src: &param, dst: &type_param, statics: &statics };
    let mut type_: syn::Type = parse_quote!(#ident #parameters);
    syn::visit_mut::visit_type_mut(&mut visitor, &mut type_);
    let schema = match cfg!(feature = "schema") {
        false => None,
        true => Some(quote! {
            fn schema(rules: &mut #wire::internal::Rules) {
                #(#schema)*
            }
        }),
    };
    let impl_wire: syn::ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #generics #wire::internal::Wire<#param> for #ident #parameters #where_clause {
            type Type<#type_param> = #type_;
            #schema
            fn encode(
                &self, writer: &mut #wire::internal::Writer<#param>
            ) -> #wire::internal::Result<()> {
                #(#encode)*
            }
            fn decode(
                reader: &mut #wire::internal::Reader<#param>
            ) -> #wire::internal::Result<Self> {
                #(#decode)*
            }
        }
    };
    Ok(impl_wire)
}

fn derive_struct(
    attrs: &mut Attrs, wire: &syn::Path, param: &syn::Lifetime, name: &syn::Ident,
    item: &syn::DataStruct,
) -> syn::Result<(Vec<syn::Stmt>, Vec<syn::Stmt>, Vec<syn::Stmt>)> {
    let mut types = Types::default();
    if item.fields.len() == 1
        && let Attr::Present(_, refine) = attrs.refine.take()
    {
        let inner = item.fields.iter().next().unwrap();
        let ty = &inner.ty;
        let schema = parse_quote! {
            rules.alias::<
                Self::Type<'static>, <#ty as #wire::internal::Wire<#param>>::Type<'static>>();
        };
        let encode: syn::Expr = match &inner.ident {
            None => parse_quote!(self.0.encode(writer)),
            Some(name) => parse_quote!(self.#name.encode(writer)),
        };
        let decode: syn::Expr = parse_quote!(#refine(<#ty>::decode(reader)?));
        return Ok((
            vec![schema],
            vec![syn::Stmt::Expr(encode, None)],
            vec![syn::Stmt::Expr(decode, None)],
        ));
    }
    let (mut schema, mut encode, decode) =
        derive_fields(wire, &parse_quote!(#name), &item.fields, &mut types)?;
    if cfg!(feature = "schema") {
        types.stmts(&mut schema, wire, "struct_", "fields");
    }
    if !encode.is_empty() {
        let path = parse_quote!(#name);
        let encode_pat = fields_pat(&path, &item.fields, false);
        encode.insert(0, parse_quote!(let #encode_pat = self;));
    }
    Ok((schema, encode, decode))
}

fn derive_enum(
    attrs: &mut Attrs, wire: &syn::Path, name: &syn::Ident, item: &syn::DataEnum,
) -> syn::Result<(Vec<syn::Stmt>, Vec<syn::Stmt>, Vec<syn::Stmt>)> {
    let mut schema = Vec::new();
    let mut types = Types::default();
    let mut encode = Vec::<syn::Arm>::new();
    let mut decode = Vec::<syn::Arm>::new();
    let mut tags = Tags::default();
    match item.variants.len() {
        _ if !cfg!(feature = "schema") => (),
        0 => schema.push(parse_quote!(let variants = #wire::internal::Vec::new();)),
        n => schema.push(parse_quote!(let mut variants = #wire::internal::Vec::with_capacity(#n);)),
    }
    for variant in &item.variants {
        let mut attrs = Attrs::parse(&variant.attrs, AttrsKind::Variant)?;
        let tag = match attrs.tag.take() {
            Attr::Present(span, tag) => tags.use_(span, tag)?,
            Attr::Absent => tags.next(),
        };
        let ident = &variant.ident;
        let path = parse_quote!(#name::#ident);
        let (variant_schema, variant_encode, variant_decode) =
            derive_fields(wire, &path, &variant.fields, &mut types)?;
        let pat_encode = fields_pat(&path, &variant.fields, true);
        let ident_schema = format!("{ident}");
        if cfg!(feature = "schema") {
            schema.push(parse_quote!({
                #(#variant_schema)*
                variants.push((#ident_schema, #tag, fields));
            }));
        }
        encode.push(parse_quote!(#pat_encode => {
            #wire::internal::encode_tag(#tag, writer)?;
            #(#variant_encode)*
        }));
        decode.push(parse_quote!(#tag => { #(#variant_decode)* }));
    }
    if let Attr::Present(span, range) = attrs.range.take()
        && (tags.used.len() as u32 != range || tags.used.iter().any(|x| range <= *x))
    {
        return Err(syn::Error::new(span, "tags don't form a range"));
    }
    if cfg!(feature = "schema") {
        types.stmts(&mut schema, wire, "enum_", "variants");
    }
    let encode = parse_quote!(match *self { #(#encode)* });
    let decode = parse_quote! {
        let tag = #wire::internal::decode_tag(reader)?;
        match tag {
            #(#decode)*
            _ => Err(#wire::internal::INVALID_TAG),
        }
    };
    Ok((schema, encode, decode))
}

fn derive_fields<'a>(
    wire: &syn::Path, name: &syn::Path, fields: &'a syn::Fields, types: &mut Types<'a>,
) -> syn::Result<(Vec<syn::Stmt>, Vec<syn::Stmt>, Vec<syn::Stmt>)> {
    let mut schema = Vec::new();
    let mut encode = Vec::new();
    let mut decode = Vec::new();
    match fields.len() {
        _ if !cfg!(feature = "schema") => (),
        0 => schema.push(parse_quote!(let fields = #wire::internal::Vec::new();)),
        n => schema.push(parse_quote!(let mut fields = #wire::internal::Vec::with_capacity(#n);)),
    }
    for (i, field) in fields.iter().enumerate() {
        let _ = Attrs::parse(&field.attrs, AttrsKind::Invalid)?;
        let name = field_name(i, field);
        let ty = &field.ty;
        if cfg!(feature = "schema") {
            let name_str: syn::Expr = match &field.ident {
                Some(x) => {
                    let x = format!("{x}");
                    parse_quote!(Some(#x))
                }
                None => parse_quote!(None),
            };
            schema.push(parse_quote!(fields.push((#name_str, #wire::internal::type_id::<#ty>()));));
            types.insert(ty);
        }
        encode.push(parse_quote!(<#ty as #wire::internal::Wire>::encode(#name, writer)?;));
        decode.push(parse_quote!(let #name = <#ty as #wire::internal::Wire>::decode(reader)?;));
    }
    encode.push(syn::Stmt::Expr(parse_quote!(Ok(())), None));
    let fields_pat = fields_pat(name, fields, false);
    decode.push(syn::Stmt::Expr(parse_quote!(Ok(#fields_pat)), None));
    Ok((schema, encode, decode))
}

fn fields_pat(name: &syn::Path, fields: &syn::Fields, ref_: bool) -> syn::Pat {
    let ref_: Option<syn::Token![ref]> = ref_.then_some(parse_quote!(ref));
    let names = fields.iter().enumerate().map(|(i, field)| field_name(i, field));
    match fields {
        syn::Fields::Named(_) => parse_quote!(#name { #(#ref_ #names),* }),
        syn::Fields::Unnamed(_) => parse_quote!(#name(#(#ref_ #names),*)),
        syn::Fields::Unit => parse_quote!(#name),
    }
}

fn field_name(i: usize, field: &syn::Field) -> syn::Ident {
    match &field.ident {
        Some(x) => x.clone(),
        None => syn::Ident::new(&format!("x{i}"), field.span()),
    }
}

struct MakeType<'a> {
    src: &'a syn::Lifetime,
    dst: &'a syn::Lifetime,
    statics: &'a [syn::Type],
}

impl syn::visit_mut::VisitMut for MakeType<'_> {
    fn visit_generic_argument_mut(&mut self, x: &mut syn::GenericArgument) {
        match x {
            syn::GenericArgument::Lifetime(a) if a == self.src => *a = self.dst.clone(),
            syn::GenericArgument::Type(t) if !self.statics.contains(t) => {
                let a = &self.dst;
                *x = parse_quote!(#t::Type<#a>)
            }
            _ => (),
        }
    }
}

#[derive(Default)]
struct Tags {
    used: HashSet<u32>,
    next: u32,
}

impl Tags {
    fn use_(&mut self, span: Span, tag: u32) -> syn::Result<u32> {
        if !self.used.insert(tag) {
            return Err(syn::Error::new(span, "duplicate tag"));
        }
        Ok(self.update_next(tag))
    }

    fn next(&mut self) -> u32 {
        while !self.used.insert(self.next) {
            self.next = self.next.wrapping_add(1);
        }
        self.update_next(self.next)
    }

    fn update_next(&mut self, tag: u32) -> u32 {
        self.next = tag.wrapping_add(1);
        tag
    }
}

#[derive(Default)]
struct Types<'a>(Vec<&'a syn::Type>);

impl<'a> Types<'a> {
    fn insert(&mut self, x: &'a syn::Type) {
        if !self.0.contains(&x) {
            self.0.push(x);
        }
    }

    fn stmts(&self, schema: &mut Vec<syn::Stmt>, wire: &syn::Path, fun: &str, var: &str) {
        let types: Vec<syn::Stmt> =
            self.0.iter().map(|ty| parse_quote!(#wire::internal::schema::<#ty>(rules);)).collect();
        let fun = syn::Ident::new(fun, Span::call_site());
        let var = syn::Ident::new(var, Span::call_site());
        schema.push(parse_quote! {
            if rules.#fun::<Self::Type<'static>>(#var) {
                #(#types)*
            }
        });
    }
}

enum AttrsKind {
    Item,
    Variant,
    Invalid,
}

#[derive(PartialEq, Eq)]
enum AttrKind {
    Crate,
    Param,
    Where,
    Tag,
    Static,
    Range,
}

#[derive(Default)]
enum Attr<T> {
    #[default]
    Absent,
    Present(Span, T),
}

impl<T> Attr<T> {
    fn span(&self) -> Option<Span> {
        match self {
            Attr::Absent => None,
            Attr::Present(x, _) => Some(*x),
        }
    }

    fn set(&mut self, span: Span, value: T) -> syn::Result<()> {
        match self {
            Attr::Absent => Ok(*self = Attr::Present(span, value)),
            Attr::Present(other, _) => {
                let mut error = syn::Error::new(span, "attribute already defined");
                error.combine(syn::Error::new(*other, "first attribute definition"));
                Err(error)
            }
        }
    }

    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl<T> Attr<Vec<T>> {
    fn push(&mut self, span: Span, value: T) {
        match self {
            Attr::Absent => *self = Attr::Present(span, vec![value]),
            Attr::Present(_, values) => values.push(value),
        }
    }
}

#[derive(Default)]
struct Attrs {
    crate_: Attr<syn::Path>,
    param: Attr<syn::Lifetime>,
    where_: Attr<Vec<syn::WherePredicate>>,
    tag: Attr<u32>,
    static_: Attr<Vec<syn::Ident>>,
    range: Attr<u32>,
    refine: Attr<syn::Path>,
}

impl Attrs {
    fn parse(attrs: &[syn::Attribute], kind: AttrsKind) -> syn::Result<Self> {
        let mut result = Attrs::default();
        for attr in attrs {
            result.parse_attr(attr)?;
        }
        result.check_kind(kind)?;
        Ok(result)
    }

    fn parse_attr(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if !attr.path().is_ident("wire") {
            return Ok(());
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                // #[wire(crate = <path>)]
                self.crate_.set(attr.span(), meta.value()?.parse()?)?;
            }
            if meta.path.is_ident("param") {
                // #[wire(param = <lifetime>)]
                self.param.set(attr.span(), meta.value()?.parse()?)?;
            }
            if meta.path.is_ident("where") {
                // #[wire(where = <where_predicate>)]
                self.where_.push(attr.span(), meta.value()?.parse()?);
            }
            if meta.path.is_ident("tag") {
                // #[wire(tag = <u32>)]
                let tag: syn::LitInt = meta.value()?.parse()?;
                self.tag.set(attr.span(), tag.base10_parse()?)?;
            }
            if meta.path.is_ident("static") {
                // #[wire(static = <ident>)]
                self.static_.push(attr.span(), meta.value()?.parse()?);
            }
            if meta.path.is_ident("range") {
                // #[wire(range = <u32>)]
                let range: syn::LitInt = meta.value()?.parse()?;
                self.range.set(attr.span(), range.base10_parse()?)?;
            }
            if meta.path.is_ident("refine") {
                // #[wire(refine = <path>)]
                self.refine.set(attr.span(), meta.value()?.parse()?)?;
            }
            Ok(())
        })
    }

    fn check_kind(&self, kind: AttrsKind) -> syn::Result<()> {
        let expected: &[AttrKind] = match kind {
            AttrsKind::Item => &[
                AttrKind::Crate,
                AttrKind::Param,
                AttrKind::Where,
                AttrKind::Static,
                AttrKind::Range,
            ],
            AttrsKind::Variant => &[AttrKind::Tag],
            AttrsKind::Invalid => &[],
        };
        let check = |name, actual, expected: &[AttrKind]| {
            if let Some(actual) = actual
                && !expected.contains(&name)
            {
                return Err(syn::Error::new(actual, "unexpected attribute"));
            }
            Ok(())
        };
        check(AttrKind::Crate, self.crate_.span(), expected)?;
        check(AttrKind::Param, self.param.span(), expected)?;
        check(AttrKind::Where, self.where_.span(), expected)?;
        check(AttrKind::Tag, self.tag.span(), expected)?;
        check(AttrKind::Static, self.static_.span(), expected)?;
        check(AttrKind::Range, self.range.span(), expected)?;
        Ok(())
    }
}
