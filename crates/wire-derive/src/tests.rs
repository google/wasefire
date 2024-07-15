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

use similar_asserts::SimpleDiff;
use syn::parse_quote;

use crate::derive_item;

#[track_caller]
fn test_ok(item: syn::DeriveInput, mut expected: syn::ItemImpl) {
    let actual = derive_item(&item).unwrap();
    expected.attrs.push(parse_quote!(#[automatically_derived]));
    #[cfg(not(feature = "schema"))]
    let _ = expected.items.remove(1);
    if actual == expected {
        return;
    }
    print!("{}", format(item));
    let actual_str = format(actual.clone());
    let expected_str = format(expected.clone());
    if actual_str == expected_str {
        similar_asserts::assert_eq!(expected, actual);
        unreachable!();
    }
    panic!("{}", SimpleDiff::from_str(&expected_str, &actual_str, "expected", "actual"));
}

fn format(item: impl Into<syn::Item>) -> String {
    prettyplease::unparse(&syn::File { shebang: None, attrs: vec![], items: vec![item.into()] })
}

macro_rules! test_ok {
    ($(#[$m:meta])* fn $name:ident() {} $input:item $output:item) => {
        $(#[$m])* #[test]
        fn $name() {
            test_ok(parse_quote!($input), parse_quote!($output));
        }
    };
}

test_ok! {
    fn unit_struct() {}
    struct Foo;
    impl<'wire> wasefire_wire::internal::Wire<'wire> for Foo {
        type Type<'_wire> = Foo;
        fn schema(rules: &mut wasefire_wire::internal::Rules) {
            let fields = wasefire_wire::internal::Vec::new();
            if rules.struct_::<Self::Type<'static>>(fields) {}
        }
        fn encode(&self, writer: &mut wasefire_wire::internal::Writer<'wire>) -> wasefire_wire::internal::Result<()> {
            let Foo = self;
            Ok(())
        }
        fn decode(reader: &mut wasefire_wire::internal::Reader<'wire>) -> wasefire_wire::internal::Result<Self> {
            Ok(Foo)
        }
    }
}

test_ok! {
    fn empty_enum() {}
    #[wire(crate = wire)]
    enum Foo {}
    impl<'wire> wire::internal::Wire<'wire> for Foo {
        type Type<'_wire> = Foo;
        fn schema(rules: &mut wire::internal::Rules) {
            let variants = wire::internal::Vec::new();
            if rules.enum_::<Self::Type<'static>>(variants) {}
        }
        fn encode(&self, writer: &mut wire::internal::Writer<'wire>) -> wire::internal::Result<()> {
            match *self {}
        }
        fn decode(reader: &mut wire::internal::Reader<'wire>) -> wire::internal::Result<Self> {
            let tag = wire::internal::decode_tag(reader)?;
            match tag {
                _ => Err(wire::internal::INVALID_TAG),
            }
        }
    }
}

test_ok! {
    fn struct_owned_named() {}
    #[wire(crate = wire)]
    struct Foo {
        bar: u8,
        baz: u32,
    }
    impl<'wire> wire::internal::Wire<'wire> for Foo {
        type Type<'_wire> = Foo;
        fn schema(rules: &mut wire::internal::Rules) {
            let mut fields = wire::internal::Vec::with_capacity(2usize);
            fields.push((Some("bar"), wire::internal::type_id::<u8>()));
            fields.push((Some("baz"), wire::internal::type_id::<u32>()));
            if rules.struct_::<Self::Type<'static>>(fields) {
                wire::internal::schema::<u8>(rules);
                wire::internal::schema::<u32>(rules);
            }
        }
        fn encode(&self, writer: &mut wire::internal::Writer<'wire>) -> wire::internal::Result<()> {
            let Foo { bar, baz } = self;
            <u8 as wire::internal::Wire>::encode(bar, writer)?;
            <u32 as wire::internal::Wire>::encode(baz, writer)?;
            Ok(())
        }
        fn decode(reader: &mut wire::internal::Reader<'wire>) -> wire::internal::Result<Self> {
            let bar = <u8 as wire::internal::Wire>::decode(reader)?;
            let baz = <u32 as wire::internal::Wire>::decode(reader)?;
            Ok(Foo { bar, baz })
        }
    }
}

test_ok! {
    fn option() {}
    #[wire(crate = wire, where = T: Wire<'wire>)]
    enum Option<T> {
        None,
        Some(T),
    }
    impl<'wire, T> wire::internal::Wire<'wire> for Option<T> where T: Wire<'wire> {
        type Type<'_wire> = Option<T::Type<'_wire>>;
        fn schema(rules: &mut wire::internal::Rules) {
            let mut variants = wire::internal::Vec::with_capacity(2usize);
            {
                let fields = wire::internal::Vec::new();
                variants.push(("None", 0u32, fields));
            }
            {
                let mut fields = wire::internal::Vec::with_capacity(1usize);
                fields.push((None, wire::internal::type_id::<T>()));
                variants.push(("Some", 1u32, fields));
            }
            if rules.enum_::<Self::Type<'static>>(variants) {
                wire::internal::schema::<T>(rules);
            }
        }
        fn encode(&self, writer: &mut wire::internal::Writer<'wire>) -> wire::internal::Result<()> {
            match *self {
                Option::None => {
                    wire::internal::encode_tag(0u32, writer)?;
                    Ok(())
                }
                Option::Some(ref x0) => {
                    wire::internal::encode_tag(1u32, writer)?;
                    <T as wire::internal::Wire>::encode(x0, writer)?;
                    Ok(())
                }
            }
        }
        fn decode(reader: &mut wire::internal::Reader<'wire>) -> wire::internal::Result<Self> {
            let tag = wire::internal::decode_tag(reader)?;
            match tag {
                0u32 => {
                    Ok(Option::None)
                }
                1u32 => {
                    let x0 = <T as wire::internal::Wire>::decode(reader)?;
                    Ok(Option::Some(x0))
                }
                _ => Err(wire::internal::INVALID_TAG),
            }
        }
    }
}

test_ok! {
    fn result() {}
    #[wire(crate = wire, where = T: Wire<'wire>, where = E: Wire<'wire>, static = E)]
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }
    impl<'wire, T, E> wire::internal::Wire<'wire> for Result<T, E>
    where T: Wire<'wire>, E: Wire<'wire>
    {
        type Type<'_wire> = Result<T::Type<'_wire>, E>;
        fn schema(rules: &mut wire::internal::Rules) {
            let mut variants = wire::internal::Vec::with_capacity(2usize);
            {
                let mut fields = wire::internal::Vec::with_capacity(1usize);
                fields.push((None, wire::internal::type_id::<T>()));
                variants.push(("Ok", 0u32, fields));
            }
            {
                let mut fields = wire::internal::Vec::with_capacity(1usize);
                fields.push((None, wire::internal::type_id::<E>()));
                variants.push(("Err", 1u32, fields));
            }
            if rules.enum_::<Self::Type<'static>>(variants) {
                wire::internal::schema::<T>(rules);
                wire::internal::schema::<E>(rules);
            }
        }
        fn encode(&self, writer: &mut wire::internal::Writer<'wire>) -> wire::internal::Result<()> {
            match *self {
                Result::Ok(ref x0) => {
                    wire::internal::encode_tag(0u32, writer)?;
                    <T as wire::internal::Wire>::encode(x0, writer)?;
                    Ok(())
                }
                Result::Err(ref x0) => {
                    wire::internal::encode_tag(1u32, writer)?;
                    <E as wire::internal::Wire>::encode(x0, writer)?;
                    Ok(())
                }
            }
        }
        fn decode(reader: &mut wire::internal::Reader<'wire>) -> wire::internal::Result<Self> {
            let tag = wire::internal::decode_tag(reader)?;
            match tag {
                0u32 => {
                    let x0 = <T as wire::internal::Wire>::decode(reader)?;
                    Ok(Result::Ok(x0))
                }
                1u32 => {
                    let x0 = <E as wire::internal::Wire>::decode(reader)?;
                    Ok(Result::Err(x0))
                }
                _ => Err(wire::internal::INVALID_TAG),
            }
        }
    }
}

test_ok! {
    fn enum_owned_mixed() {}
    #[wire(crate = wire)]
    enum Foo {
        #[wire(tag = 1)]
        Mid(u8, u8),
        #[wire(tag = 2)]
        Last { bar: u8 },
        #[wire(tag = 0)]
        First,
    }
    impl<'wire> wire::internal::Wire<'wire> for Foo {
        type Type<'_wire> = Foo;
        fn schema(rules: &mut wire::internal::Rules) {
            let mut variants = wire::internal::Vec::with_capacity(3usize);
            {
                let mut fields = wire::internal::Vec::with_capacity(2usize);
                fields.push((None, wire::internal::type_id::<u8>()));
                fields.push((None, wire::internal::type_id::<u8>()));
                variants.push(("Mid", 1u32, fields));
            }
            {
                let mut fields = wire::internal::Vec::with_capacity(1usize);
                fields.push((Some("bar"), wire::internal::type_id::<u8>()));
                variants.push(("Last", 2u32, fields));
            }
            {
                let fields = wire::internal::Vec::new();
                variants.push(("First", 0u32, fields));
            }
            if rules.enum_::<Self::Type<'static>>(variants) {
                wire::internal::schema::<u8>(rules);
            }
        }
        fn encode(&self, writer: &mut wire::internal::Writer<'wire>) -> wire::internal::Result<()> {
            match *self {
                Foo::Mid(ref x0, ref x1) => {
                    wire::internal::encode_tag(1u32, writer)?;
                    <u8 as wire::internal::Wire>::encode(x0, writer)?;
                    <u8 as wire::internal::Wire>::encode(x1, writer)?;
                    Ok(())
                }
                Foo::Last { ref bar } => {
                    wire::internal::encode_tag(2u32, writer)?;
                    <u8 as wire::internal::Wire>::encode(bar, writer)?;
                    Ok(())
                }
                Foo::First => {
                    wire::internal::encode_tag(0u32, writer)?;
                    Ok(())
                }
            }
        }
        fn decode(reader: &mut wire::internal::Reader<'wire>) -> wire::internal::Result<Self> {
            let tag = wire::internal::decode_tag(reader)?;
            match tag {
                1u32 => {
                    let x0 = <u8 as wire::internal::Wire>::decode(reader)?;
                    let x1 = <u8 as wire::internal::Wire>::decode(reader)?;
                    Ok(Foo::Mid(x0, x1))
                }
                2u32 => {
                    let bar = <u8 as wire::internal::Wire>::decode(reader)?;
                    Ok(Foo::Last { bar })
                }
                0u32 => {
                    Ok(Foo::First)
                }
                _ => Err(wire::internal::INVALID_TAG),
            }
        }
    }
}
