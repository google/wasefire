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

use std::borrow::Cow;
use std::time::Duration;

use data_encoding::HEXLOWER as HEX;
use futures_util::StreamExt;
use gloo::file::File;
use wasefire_common::platform::Side;
use wasefire_error::Code;
use wasefire_protocol::applet::AppletId;
use wasefire_protocol::{self as service, ConnectionExt as _, Service, transfer};
use webusb_web::UsbDevice;
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::{AttrValue, Callback, Html, UseStateHandle, UseStateSetter, html};

use crate::usb::{Device, serial_number};

pub(crate) enum Page {
    Error { error: String, device: Option<Device> },
    Feedback { content: Html },
    Result { content: Html, device: Option<Device> },
    ListDevices,
    ChooseDevice { devices: Vec<UsbDevice> },
    RequestDevice,
    OpenDevice { device: UsbDevice },
    ForgetDevice { device: UsbDevice },
    Connected { device: Device },
}

pub(crate) fn render(page: UseStateHandle<Page>) -> Html {
    let close = |device: Option<Device>| {
        Callback::from({
            let page = page.setter();
            move |_| match device.clone() {
                None => page.set(Page::ListDevices),
                Some(device) => page.set(Page::Connected { device }),
            }
        })
    };
    match &*page {
        Page::Error { error, device } => {
            let close = close(device.clone());
            html_error(html!(<>{ error }{ " " }<button onclick={close}>{ "Close" }</button></>))
        }
        Page::Feedback { content } => html_running(content.clone()),
        Page::Result { content, device } => {
            let close = close(device.clone());
            html_idle(html!(<>{ content.clone() }<button onclick={close}>{ "Close" }</button></>))
        }
        Page::ListDevices => {
            spawn_local(async move {
                match crate::usb::list_devices().await {
                    Ok(devices) => page.set(Page::ChooseDevice { devices }),
                    Err(error) => page.set(Page::error(error)),
                }
            });
            html_running("Looking for paired USB devices...".into())
        }
        Page::ChooseDevice { devices } => {
            let refresh = Callback::from({
                let page = page.setter();
                move |_| page.set(Page::ListDevices)
            });
            let request = Callback::from({
                let page = page.setter();
                move |_| page.set(Page::RequestDevice)
            });
            let device_list = devices.iter().map(|device| {
                let connect = Callback::from({
                    let page = page.setter();
                    let device = device.clone();
                    move |_| page.set(Page::OpenDevice { device: device.clone() })
                });
                let forget = Callback::from({
                    let page = page.setter();
                    let device = device.clone();
                    move |_| page.set(Page::ForgetDevice { device: device.clone() })
                });
                html! {
                    <li>
                        <button onclick={connect}>{ "Connect" }</button>{ " to device " }
                        <code>{ serial_number(device) }</code>{ " or " }
                        <button onclick={forget}>{ "Forget" }</button>{ " it." }
                    </li>
                }
            });
            html_idle(html! {<>
                if devices.is_empty() {
                    { "No paired devices found. " }
                } else {
                    { "List of already paired device:" }
                    <ul class="devices">{ for device_list }</ul>
                }
                <button onclick={refresh}>{ "Refresh" }</button>{ " the list or " }
                <button onclick={request}>{ "Pair" }</button>{ " a new device." }
            </>})
        }
        Page::RequestDevice => {
            spawn_local(async move {
                match crate::usb::request_device().await {
                    Ok(device) => page.set(Page::OpenDevice { device }),
                    Err(error) => page.set(Page::error(error)),
                }
            });
            html_running("Pairing a new device...".into())
        }
        Page::OpenDevice { device } => {
            let device = device.clone();
            spawn_local(async move {
                match crate::usb::open_device(&device).await {
                    Ok(device) => page.set(Page::Connected { device }),
                    Err(error) => page.set(Page::error(error)),
                }
            });
            html_running("Connecting to device...".into())
        }
        Page::ForgetDevice { device } => {
            let device = device.clone();
            spawn_local(async move {
                crate::usb::forget_device(device).await;
                gloo::utils::window().location().reload().unwrap();
            });
            html_running("Forgetting device...".into())
        }
        Page::Connected { device } => {
            let disconnect = Callback::from({
                let page = page.setter();
                move |_| page.set(Page::ListDevices)
            });
            let platform = [
                service::PlatformInfo::input(page.setter(), device.clone()),
                service::PlatformReboot::input(page.setter(), device.clone()),
                service::PlatformUpdate::input(page.setter(), device.clone()),
            ];
            let applet = [
                service::AppletExitStatus::input(page.setter(), device.clone()),
                service::AppletReboot::input(page.setter(), device.clone()),
                service::AppletInstall::input(page.setter(), device.clone()),
            ];
            html_idle(html! {<>
                <div class="columns">
                    <div class="column">{ "Platform operations:" }
                        <ul class="commands">{ for platform }</ul></div>
                    <div class="column">{ "Applet operations:" }
                        <ul class="commands">{ for applet }</ul></div>
                </div>
                <button onclick={disconnect}>{ "Disconnect" }</button>{ " from device " }
                <code>{ serial_number(device.device()) }</code>{ "." }
                <footer>{ format!("Protocol version: {}", device.version()) }</footer>
            </>})
        }
    }
}

impl Page {
    fn error(error: impl ToString) -> Self {
        Page::Error { error: error.to_string(), device: None }
    }

    fn error_device(error: impl ToString, device: &Device) -> Self {
        Page::Error { error: error.to_string(), device: Some(device.clone()) }
    }
}

fn html_error(content: Html) -> Html {
    html!(<div class="error">{ content }</div>)
}

fn html_running(content: Html) -> Html {
    html!(<div class="running">{ content }</div>)
}

fn html_idle(content: Html) -> Html {
    html!(<div class="idle">{ content }</div>)
}

macro_rules! unwrap {
    ($p:expr, $d:expr, $x:expr) => {
        match $x {
            Ok(x) => x,
            Err(e) => {
                let (error, device) = match e.downcast_ref::<wasefire_error::Error>() {
                    None => (e.to_string(), None),
                    Some(e) => (e.to_string(), Some($d.clone())),
                };
                $p.set(Page::Error { error, device });
                return Default::default();
            }
        }
    };
}

fn convert_if<T, E: std::error::Error + Send + Sync + 'static>(
    x: anyhow::Result<T>, p: impl Fn(&E) -> bool,
) -> anyhow::Result<Option<T>> {
    match x {
        Ok(x) => Ok(Some(x)),
        Err(e) if e.downcast_ref().is_some_and(p) => Ok(None),
        Err(e) => Err(e),
    }
}

fn convert_final<T>(x: anyhow::Result<T>) -> anyhow::Result<Option<T>> {
    convert_if::<_, webusb_web::Error>(x, crate::usb::is_transfer_in_error)
}

fn convert_not_found<T>(x: anyhow::Result<T>) -> anyhow::Result<Option<T>> {
    convert_if::<_, wasefire_error::Error>(x, |e| e == &wasefire_error::Error::user(Code::NotFound))
}

trait Command: Service {
    fn input(page: UseStateSetter<Page>, device: Device) -> Html;
}

impl Command for service::PlatformInfo {
    fn input(page: UseStateSetter<Page>, device: Device) -> Html {
        let click = Callback::from(move |_| {
            page.set(Page::Feedback { content: "Requesting platform info...".into() });
            let page = page.clone();
            let device = device.clone();
            spawn_local(async move {
                let content = platform_info(unwrap!(page, device, device.platform_info().await));
                page.set(Page::Result { content, device: Some(device) });
            });
        });
        html!(<li><button onclick={click}>{ "Read" }</button>{ " platform info" }</li>)
    }
}

fn platform_info(info: wasefire_protocol::platform::DynInfo) -> Html {
    let mut elements = Vec::new();
    elements.push(html!(<li>{ "Serial number: " }<code>{ HEX.encode(info.serial()) }</code></li>));
    if let Some(applet) = info.applet_kind() {
        elements.push(html!(<li>{ "Applet kind: " }<code>{ applet.to_string() }</code></li>));
    }
    if let Some(side) = info.running_side() {
        elements.push(html!(<li>{ "Running side: " }<code>{ side.to_string() }</code></li>));
    }
    if let Some(name_hex) = info.running_name() {
        let mut element = Vec::new();
        element.push("Running name: ".into());
        element.push(html!(<code>{ HEX.encode(name_hex) }</code>));
        if let Some(name_str) = wasefire_protocol::platform::name_str(name_hex) {
            element.push(" (".into());
            element.push(html!(<code>{ name_str }</code>));
            element.push(")".into());
        }
        elements.push(html!(<li>{ for element }</li>));
    }
    elements.push(
        html!(<li>{ "Running version: " }<code>{ HEX.encode(info.running_version()) }</code></li>),
    );
    fn push<T>(
        elements: &mut Vec<Html>, title: &str, value: Option<Result<T, wasefire_error::Error>>,
        action: impl FnOnce(T) -> Html,
    ) {
        let Some(value) = value else { return };
        let value = match value {
            Ok(x) => action(x),
            Err(e) => format!("{e}").into(),
        };
        elements.push(html!(<li>{ format!("Opposite {title}: ") }{ value }</li>));
    }
    push(&mut elements, "name", info.opposite_name(), |x| {
        let mut element = Vec::new();
        element.push(html!(<code>{ HEX.encode(x) }</code>));
        if let Some(x) = wasefire_protocol::platform::name_str(x) {
            element.push(" (".into());
            element.push(html!(<code>{ x }</code>));
            element.push(")".into());
        }
        Html::from_iter(element)
    });
    push(
        &mut elements,
        "version",
        info.opposite_version(),
        |x| html!(<code>{ HEX.encode(x) }</code>),
    );
    html!(<>{ "Platform info:" }<ul>{ for elements }</ul></>)
}

impl Command for service::PlatformReboot {
    fn input(page: UseStateSetter<Page>, device: Device) -> Html {
        let click = Callback::from(move |_| {
            page.set(Page::Feedback { content: "Rebooting platform...".into() });
            let page = page.clone();
            let device = device.clone();
            spawn_local(async move {
                let reboot = device.call::<service::PlatformReboot>(()).await;
                match unwrap!(page, device, convert_final(reboot)) {
                    Some(never) => *never.get(),
                    None => platform_reboot(page, device.device().clone()).await,
                }
            });
        });
        html!(<li><button onclick={click}>{ "Reboot" }</button>{ " platform" }</li>)
    }
}

async fn platform_reboot(page: UseStateSetter<Page>, device: UsbDevice) {
    for _ in 0 .. 10 {
        if !device.opened() {
            return page.set(Page::Result { content: "Platform rebooted. ".into(), device: None });
        }
        sleep(Duration::from_millis(100)).await;
    }
    page.set(Page::error("Reboot seems to have failed."));
}

impl Command for service::AppletInstall {
    fn input(page: UseStateSetter<Page>, device: Device) -> Html {
        let uninstall = Callback::from({
            let page = page.clone();
            let device = device.clone();
            move |_| {
                let page = page.clone();
                let device = device.clone();
                spawn_local(async move {
                    if transfer::<service::AppletInstall>(&page, &device, &[], None).await {
                        let content = "Applet uninstalled. ".into();
                        page.set(Page::Result { content, device: Some(device) });
                    }
                });
            }
        });
        html! {<>
            <li><AppletInstall page={page} device={device} /></li>
            <li><button onclick={uninstall}>{ "Uninstall" }</button>{ " applet" }</li>
        </>}
    }
}

#[yew_autoprops::autoprops(AppletInstallProps)]
#[yew::component(AppletInstall)]
fn applet_install(page: UseStateSetter<Page>, device: Device) -> Html {
    let file = yew::use_node_ref();
    let install = Callback::from({
        let file = file.clone();
        move |_| {
            let node = file.cast::<web_sys::HtmlInputElement>().unwrap();
            let Some(file) = node.files().and_then(|x| x.item(0)) else {
                return page.set(Page::error_device("No file selected.", &device));
            };
            page.set(Page::Feedback { content: "Reading file...".into() });
            let file = File::from(file);
            let page = page.clone();
            let device = device.clone();
            spawn_local(async move {
                let content = match gloo::file::futures::read_as_bytes(&file).await {
                    Ok(x) => x,
                    Err(error) => return page.set(Page::error_device(error, &device)),
                };
                if transfer::<service::AppletInstall>(&page, &device, &content, None).await {
                    let content = "Applet installed. ".into();
                    page.set(Page::Result { content, device: Some(device) });
                }
            });
        }
    });
    html! {<>
        <button onclick={install}>{ "Install" }</button>{ " applet:" }
        <ul class="commands"><li><input ref={file} type="file" /></li></ul>
    </>}
}

async fn transfer<
    T: for<'a> Service<Request<'a> = transfer::Request<'a>, Response<'a> = transfer::Response>,
>(
    page: &UseStateSetter<Page>, device: &Device, content: &[u8], kind: Option<bool>,
) -> bool {
    let title = match kind {
        None if content.is_empty() => AttrValue::Static("Uninstalling applet"),
        None => AttrValue::Static("Installing applet"),
        Some(false) => AttrValue::Static("Updating platform (side 1 of 2)"),
        Some(true) => AttrValue::Static("Updating platform (side 2 of 2)"),
    };
    page.set(Page::Feedback { content: html!(<h2>{ &title }</h2>) });
    let start = device.call::<T>(transfer::Request::Start { dry_run: false }).await;
    let start = unwrap!(page, device, start);
    let transfer::Response::Start { chunk_size, num_pages } = start.get() else {
        page.set(Page::error_device("Invalid transfer response for Start.", device));
        return false;
    };
    let n = *num_pages;
    for i in 0 .. n {
        page.set(Page::Feedback {
            content: html! {<>
                <h2>{ &title }</h2>
                { "Erasing: " }<progress value={i.to_string()} max={n.to_string()}></progress>
            </>},
        });
        let erase = device.call::<T>(transfer::Request::Erase).await;
        let transfer::Response::Erase = unwrap!(page, device, erase).get() else {
            page.set(Page::error_device("Invalid transfer response for Erase.", device));
            return false;
        };
    }
    let len = AttrValue::from(content.len().to_string());
    for (i, chunk) in content.chunks(*chunk_size).enumerate() {
        let value = (*chunk_size * i).to_string();
        page.set(Page::Feedback {
            content: html! {<>
                <h2>{ &title }</h2>{ "Writing: " }<progress value={value} max={&len}></progress>
            </>},
        });
        let chunk = Cow::Borrowed(chunk);
        let write = device.call::<T>(transfer::Request::Write { chunk }).await;
        let transfer::Response::Write = unwrap!(page, device, write).get() else {
            page.set(Page::error_device("Invalid transfer response for Write.", device));
            return false;
        };
    }
    let finish = device.call::<T>(transfer::Request::Finish).await;
    let result = match kind {
        None => matches!(unwrap!(page, device, finish).get(), transfer::Response::Finish),
        Some(_) => unwrap!(page, device, convert_final(finish)).is_none(),
    };
    if !result {
        page.set(Page::error_device("Invalid transfer response for Finish.", device));
    }
    result
}

impl Command for service::AppletExitStatus {
    fn input(page: UseStateSetter<Page>, device: Device) -> Html {
        let click = Callback::from(move |_| {
            page.set(Page::Feedback { content: "Requesting applet status...".into() });
            let page = page.clone();
            let device = device.clone();
            spawn_local(async move {
                let status = device.call::<service::AppletExitStatus>(AppletId).await;
                let content = match unwrap!(page, device, convert_not_found(status)) {
                    None => "There is no applet installed. ".into(),
                    Some(x) => match x.get() {
                        None => "The applet is running. ".into(),
                        Some(x) => format!("{x}. ").into(),
                    },
                };
                page.set(Page::Result { content, device: Some(device) });
            });
        });
        html!(<li><button onclick={click}>{ "Read" }</button>{ " applet status" }</li>)
    }
}

impl Command for service::AppletReboot {
    fn input(page: UseStateSetter<Page>, device: Device) -> Html {
        let click = Callback::from(move |_| {
            page.set(Page::Feedback { content: "Rebooting applet...".into() });
            let page = page.clone();
            let device = device.clone();
            spawn_local(async move {
                let reboot = device.call::<service::AppletReboot>(AppletId).await;
                unwrap!(page, device, reboot).get();
                let content = "Applet rebooted. ".into();
                page.set(Page::Result { content, device: Some(device) });
            });
        });
        html!(<li><button onclick={click}>{ "Reboot" }</button>{ " applet" }</li>)
    }
}

impl Command for service::PlatformUpdate {
    fn input(page: UseStateSetter<Page>, device: Device) -> Html {
        html!(<li><PlatformUpdate page={page} device={device} /></li>)
    }
}

#[yew_autoprops::autoprops(PlatformUpdateProps)]
#[yew::component(PlatformUpdate)]
fn platform_update(page: UseStateSetter<Page>, device: Device) -> Html {
    let side_a = yew::use_node_ref();
    let side_b = yew::use_node_ref();
    let install = Callback::from({
        let side_a = side_a.clone();
        let side_b = side_b.clone();
        move |_| {
            let side_a = side_a.cast::<web_sys::HtmlInputElement>().unwrap();
            let side_b = side_b.cast::<web_sys::HtmlInputElement>().unwrap();
            let Some(file_a) = side_a.files().and_then(|x| x.item(0)) else {
                return page.set(Page::error_device("No file selected for side A.", &device));
            };
            let Some(file_b) = side_b.files().and_then(|x| x.item(0)) else {
                return page.set(Page::error_device("No file selected for side B.", &device));
            };
            page.set(Page::Feedback { content: "Reading files...".into() });
            let file_a = File::from(file_a);
            let file_b = File::from(file_b);
            spawn_local(platform_update_(page.clone(), device.clone(), file_a, file_b));
        }
    });
    html! {<>
        <button onclick={install}>{ "Update" }</button>{ " platform:" }
        <ul class="commands">
            <li>{ "Side A: " }<input ref={side_a} type="file" /></li>
            <li>{ "Side B: " }<input ref={side_b} type="file" /></li>
        </ul>
    </>}
}

async fn platform_update_(page: UseStateSetter<Page>, device: Device, file_a: File, file_b: File) {
    let side_a = match gloo::file::futures::read_as_bytes(&file_a).await {
        Ok(x) => x,
        Err(error) => return page.set(Page::error_device(error, &device)),
    };
    let side_b = match gloo::file::futures::read_as_bytes(&file_b).await {
        Ok(x) => x,
        Err(error) => return page.set(Page::error_device(error, &device)),
    };
    let info1 = unwrap!(page, device, device.platform_info().await);
    let Some(side1) = info1.running_side() else {
        return page.set(Page::error_device("Device does not expose running side.", &device));
    };
    let (side_1, side_2) = match side1 {
        Side::A => (side_b, side_a),
        Side::B => (side_a, side_b),
    };
    if !transfer::<service::PlatformUpdate>(&page, &device, &side_1, Some(false)).await {
        return;
    }
    page.set(Page::Feedback { content: "Updated side 1 of 2. Reconnecting...".into() });
    let device = match reconnect(device.device()).await {
        Ok(x) => x,
        Err(error) => return page.set(Page::error(error)),
    };
    let info2 = unwrap!(page, device, device.platform_info().await);
    let Some(side2) = info2.running_side() else {
        return page.set(Page::error_device("Update does not expose running side.", &device));
    };
    if side2 != side1.opposite() {
        return page.set(Page::error_device("Failed to boot the new platform.", &device));
    }
    if transfer::<service::PlatformUpdate>(&page, &device, &side_2, Some(true)).await {
        page.set(Page::Result { content: "Platform updated. ".into(), device: None });
    }
}

async fn reconnect(device: &UsbDevice) -> Result<Device, String> {
    let serial = serial_number(device);
    let usb = webusb_web::Usb::new().map_err(|x| x.to_string())?;
    let mut events = usb.events();
    while let Some(event) = events.next().await {
        let device = match event {
            webusb_web::UsbEvent::Connected(x) => x,
            _ => continue,
        };
        if crate::usb::is_wasefire(&device) && serial_number(&device) == serial {
            return crate::usb::open_device(&device).await.map_err(|x| x.to_string());
        }
    }
    Err("Stopped receiving USB events.".to_string())
}
