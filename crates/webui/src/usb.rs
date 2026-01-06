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

use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use anyhow::ensure;
use js_sys::wasm_bindgen::JsValue;
use wasefire_protocol::DynFuture;
use wasefire_protocol_usb::common::{Decoder, Encoder};
use webusb_web::{OpenUsbDevice, UsbDevice};

pub(crate) async fn list_devices() -> webusb_web::Result<Vec<UsbDevice>> {
    let mut devices = webusb_web::Usb::new()?.devices().await;
    devices.sort_by_cached_key(serial_number);
    Ok(devices)
}

pub(crate) async fn request_device() -> webusb_web::Result<UsbDevice> {
    webusb_web::Usb::new()?
        .request_device([webusb_web::UsbDeviceFilter::new()
            .with_vendor_id(VID_GOOGLE)
            .with_product_id(PID_WASEFIRE)])
        .await
}

pub(crate) async fn open_device(device: &UsbDevice) -> anyhow::Result<Device> {
    let device = device.open().await?;
    if device.device().configuration().is_none_or(|x| x.configuration_value != WASEFIRE_CONF) {
        device.select_configuration(WASEFIRE_CONF).await?;
    }
    device.claim_interface(WASEFIRE_IFACE).await?;
    Ok(Rc::new(wasefire_protocol::Device::new(Connection(device)).await?))
}

pub(crate) async fn forget_device(device: UsbDevice) {
    device.forget().await;
}

pub(crate) fn is_wasefire(device: &UsbDevice) -> bool {
    device.vendor_id() == VID_GOOGLE && device.product_id() == PID_WASEFIRE
}

pub(crate) fn serial_number(device: &UsbDevice) -> String {
    device.serial_number().unwrap_or_else(|| "N/A".to_string())
}

pub(crate) type Device = Rc<wasefire_protocol::Device<Connection>>;

pub(crate) struct Connection(OpenUsbDevice);

impl Deref for Connection {
    type Target = OpenUsbDevice;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq for Connection {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl wasefire_protocol::Connection for Connection {
    fn write<'a>(&'a self, request: &'a [u8]) -> DynFuture<'a, ()> {
        Box::pin(async move {
            for packet in Encoder::new(request) {
                let written = self.0.transfer_out(WASEFIRE_EP, &packet).await?;
                ensure!(written == MAX_PACKET_SIZE);
            }
            Ok(())
        })
    }

    fn read(&self) -> DynFuture<'_, Box<[u8]>> {
        Box::pin(async move {
            let mut decoder = Decoder::default();
            loop {
                let packet = self.0.transfer_in(WASEFIRE_EP, MAX_PACKET_SIZE).await?;
                let packet = packet.as_slice().try_into().map_err(|_| transfer_in_error())?;
                match decoder.push(packet).ok_or_else(transfer_in_error)? {
                    Ok(x) => break Ok(x.into_boxed_slice()),
                    Err(x) => decoder = x,
                }
            }
        })
    }
}

fn transfer_in_error() -> webusb_web::Error {
    let error = js_sys::Error::new("transferIn");
    error.set_name("NetworkError");
    let result = JsValue::from(error).into();
    assert!(is_transfer_in_error(&result));
    result
}

pub(crate) fn is_transfer_in_error(e: &webusb_web::Error) -> bool {
    e.kind() == webusb_web::ErrorKind::Transfer && e.msg().contains("transferIn")
}

const VID_GOOGLE: u16 = 0x18d1;
const PID_WASEFIRE: u16 = 0x0239;
const WASEFIRE_CONF: u8 = 1;
const WASEFIRE_IFACE: u8 = 0;
pub(crate) const WASEFIRE_EP: u8 = 1;
const MAX_PACKET_SIZE: u32 = 64;
