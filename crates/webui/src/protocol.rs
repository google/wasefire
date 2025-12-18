// Copyright 2025 Google LLC
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

use std::rc::Rc;

use wasefire_protocol::{ApiResult, Service};
use wasefire_protocol_usb::common::{Decoder, Encoder};
use wasefire_wire::Yoke;
use webusb_web::OpenUsbDevice;

use crate::usb::WASEFIRE_EP;

pub(crate) async fn call<'a, T: Service>(
    device: &Rc<OpenUsbDevice>, request: T::Request<'a>,
) -> Result<Yoke<T::Response<'static>>, Error> {
    let request = T::request(request).encode()?;
    for packet in Encoder::new(&request) {
        device.transfer_out(WASEFIRE_EP, &packet).await?;
    }
    let mut decoder = Decoder::default();
    let response = loop {
        let packet = device.transfer_in(WASEFIRE_EP, 64).await?;
        let packet = packet.as_slice().try_into().map_err(|_| BAD_PACKET)?;
        match decoder.push(packet).ok_or(BAD_PACKET)? {
            Ok(x) => break x,
            Err(x) => decoder = x,
        }
    };
    let response = ApiResult::<T>::decode_yoke(response.into_boxed_slice())?;
    response.try_map(|x| match x {
        ApiResult::Ok(x) => Ok(x),
        ApiResult::Err(error) => Err(error.into()),
    })
}

#[derive(Debug)]
pub(crate) enum Error {
    Usb(webusb_web::Error),
    Protocol(wasefire_error::Error),
}

impl Error {
    pub(crate) fn convert(self) -> (String, bool) {
        match self {
            Error::Usb(error) => (error.to_string(), false),
            Error::Protocol(error) => (error.to_string(), true),
        }
    }
}

impl From<webusb_web::Error> for Error {
    fn from(error: webusb_web::Error) -> Self {
        Error::Usb(error)
    }
}

impl From<wasefire_error::Error> for Error {
    fn from(error: wasefire_error::Error) -> Self {
        Error::Protocol(error)
    }
}

const BAD_PACKET: wasefire_error::Error =
    wasefire_error::Error::new_const(wasefire_error::Space::World as u8, 0);
