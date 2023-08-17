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

#[macro_export]
macro_rules! type_ {
    (usize) => (Type::Integer { signed: false, bits: None });
    (isize) => (Type::Integer { signed: true, bits: None });
    (u64) => (Type::Integer { signed: false, bits: Some(64) });
    (*const $($type:tt)*) => (type_!(*false $($type)*));
    (*mut $($type:tt)*) => (type_!(*true $($type)*));
    (*$mut:tt u8) => (Type::Pointer { mutable: $mut, type_: None });
    (*$mut:tt $($type:tt)*) => {
        Type::Pointer { mutable: $mut, type_: Some(Box::new(type_!($($type)*))) }
    };
    (fn $fields:tt) => (Type::Function { params: fields!($fields) });
}

#[macro_export]
macro_rules! docs {
    ($(#[doc = $doc:literal])*) => (vec![$($doc.into()),*]);
}

#[macro_export]
macro_rules! variants {
    ($input:tt) => (variants!($input []));

    ({ $(#[doc = $doc:literal])* $name:ident = $value:expr, $($input:tt)* }
     [$($output:tt)*]) => (variants! {
        { $($input)* } [$($output)* Variant {
            docs: vec![$($doc.into()),*],
            name: stringify!($name).into(),
            value: $value,
        },]
    });

    ({} [$($output:tt)*]) => (vec![$($output)*]);
}

#[macro_export]
macro_rules! fields {
    ($input:tt) => (fields!($input []));

    ({ $(#[doc = $doc:literal])* $(#[cfg($cfg:meta)])* $name:ident: $($input:tt)* } $output:tt
    ) => (fields! {
        [$($doc)*] [$($cfg)*] $name [] { $($input)* } $output
    });

    ($doc:tt $cfg:tt $name:ident $type:tt {} $output:tt) => (fields! {
        $doc $cfg $name $type { , } $output
    });

    ([$($doc:literal)*] [$($cfg:meta)*] $name:ident [$($type:tt)*] { , $($input:tt)* }
     [$($output:tt)*]) => (fields! {
        { $($input)* } [
            $($output)*
            $(#[cfg($cfg)])*
            Field {
                docs: vec![$($doc.into()),*],
                name: stringify!($name).into(),
                type_: type_!($($type)*),
            },
        ]
    });

    ($doc:tt $cfg:tt $name:ident [$($type:tt)*] { $x:tt $($input:tt)* } $output:tt) => (fields! {
        $doc $cfg $name [$($type)* $x] { $($input)* } $output
    });

    ({} [$($output:tt)*]) => (vec![$($output)*]);
}

#[macro_export]
macro_rules! item {
    ($(#[doc = $doc:literal])* enum $name:ident $variants:tt) => {
        Item::Enum(Enum {
            docs: vec![$($doc.into()),*],
            name: stringify!($name).into(),
            variants: variants!($variants),
        })
    };

    ($(#[doc = $doc:literal])* struct $name:ident $fields:tt) => {
        Item::Struct(Struct {
            docs: vec![$($doc.into()),*],
            name: stringify!($name).into(),
            fields: fields!($fields),
        })
    };

    ($(#[doc = $doc:literal])* fn $name:ident $link:literal $params:tt -> $results:tt) => {
        Item::Fn(Fn {
            docs: vec![$($doc.into()),*],
            name: stringify!($name).into(),
            link: $link.into(),
            params: fields!($params),
            results: fields!($results),
        })
    };
}
