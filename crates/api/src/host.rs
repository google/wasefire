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

use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::Deref;

use sealed::sealed;

/// Describes an interface function at type-level.
#[sealed(pub(crate))]
pub trait Signature: Default + Debug {
    /// Unique name for this function.
    const NAME: &'static str;

    /// The type of params for this function.
    type Params: ArrayU32;

    /// The success result type for this function.
    type Result;

    /// Returns the descriptor of this function.
    fn descriptor() -> Descriptor {
        Descriptor { name: Self::NAME, params: Self::Params::LENGTH }
    }
}

/// Describes an interface function at data-level.
#[derive(Debug, Copy, Clone)]
pub struct Descriptor {
    /// Unique name.
    pub name: &'static str,

    /// Number of (u32) params.
    pub params: usize,
}

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct U32<T> {
    value: u32,
    phantom: PhantomData<T>,
}

unsafe impl<T> Send for U32<T> {}

impl<T> Default for U32<T> {
    fn default() -> Self {
        U32 { value: 0, phantom: PhantomData }
    }
}

impl<T> Deref for U32<T> {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> From<u32> for U32<T> {
    fn from(value: u32) -> Self {
        U32 { value, phantom: PhantomData }
    }
}

impl From<bool> for U32<bool> {
    fn from(x: bool) -> Self {
        (x as u32).into()
    }
}

impl From<()> for U32<()> {
    fn from(_: ()) -> Self {
        0u32.into()
    }
}

impl From<!> for U32<!> {
    fn from(x: !) -> Self {
        match x {}
    }
}

#[sealed(pub(crate))]
pub trait ArrayU32: Sized + Copy {
    const LENGTH: usize = core::mem::size_of::<Self>() / core::mem::size_of::<u32>();

    fn from(values: &[u32]) -> &Self {
        assert_eq!(values.len(), Self::LENGTH);
        unsafe { &*(values.as_ptr() as *const Self) }
    }

    fn into(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u32, Self::LENGTH) }
    }
}

pub trait Dispatch {
    type Erased: Debug;
    type Merged<T: Signature>: Debug;
    fn merge<T: Signature>(erased: Self::Erased) -> Self::Merged<T>;
    fn erase<T: Signature>(merged: Self::Merged<T>) -> Self::Erased;
}

#[derive(Debug, Copy, Clone)]
pub struct Id;

impl Dispatch for Id {
    type Erased = ();
    type Merged<T: Signature> = T;
    fn merge<T: Signature>((): Self::Erased) -> Self::Merged<T> {
        Default::default()
    }
    fn erase<T: Signature>(_: Self::Merged<T>) -> Self::Erased {}
}
