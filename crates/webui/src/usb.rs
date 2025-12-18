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

pub(crate) async fn open_device(device: &UsbDevice) -> webusb_web::Result<OpenUsbDevice> {
    let device = device.open().await?;
    if device.device().configuration().is_none_or(|x| x.configuration_value != WASEFIRE_CONF) {
        device.select_configuration(WASEFIRE_CONF).await?;
    }
    device.claim_interface(WASEFIRE_IFACE).await?;
    Ok(device)
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

const VID_GOOGLE: u16 = 0x18d1;
const PID_WASEFIRE: u16 = 0x0239;
const WASEFIRE_CONF: u8 = 1;
const WASEFIRE_IFACE: u8 = 0;
pub(crate) const WASEFIRE_EP: u8 = 1;
