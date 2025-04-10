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

#![feature(exit_status_error)]

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{Debug, Write as _};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;

use anyhow::Result;

const OPENTITAN: &str = "../../../../../../third_party/lowRISC/opentitan";
const FASTBUILD: &str = "bazel-out/k8-fastbuild/bin";
const IP_BLOCKS: &[&str] = &[
    // Keep sorted.
    "FLASH_CTRL_CORE",
    "HMAC",
    "LC_CTRL",
    "PINMUX_AON",
    "RSTMGR_AON",
    "RV_PLIC",
    "RV_TIMER",
    "UART",
    "USBDEV",
];

fn main() -> Result<()> {
    // Build all header files.
    let mut bazel = Command::new("bazel");
    bazel.current_dir(OPENTITAN);
    bazel.arg("build");
    bazel.arg("//sw/device/silicon_owner:manifest");
    for name in IP_BLOCKS {
        bazel.arg(IpPath::build(&ip_name(name).to_lowercase()));
    }
    bazel.status()?.exit_ok()?;

    // Copy manifest.json with terminating newline.
    let mut manifest =
        std::fs::read(format!("{OPENTITAN}/{FASTBUILD}/sw/device/silicon_owner/manifest.json"))?;
    if manifest.last().is_none_or(|&x| x != b'\n') {
        manifest.push(b'\n');
    }
    std::fs::write("../../../../manifest.json", &manifest)?;

    // Create output file with copyright notice (taken from this file).
    let mut output = File::create("../../src/lib.rs")?;
    for line in BufReader::new(File::open("src/main.rs")?).lines() {
        let line = line?;
        writeln!(output, "{line}")?;
        if line.is_empty() {
            break;
        }
    }
    writeln!(output, "// This file is generated from crates/generate.\n")?;

    // Write crate-level attributes.
    writeln!(output, "#![no_std]")?;
    writeln!(output, "#![allow(clippy::erasing_op)]")?;
    writeln!(output, "#![allow(clippy::identity_op)]")?;

    // Parse instances and interrupts.
    let input = File::open(format!("{OPENTITAN}/hw/top_earlgrey/sw/autogen/top_earlgrey.h"))?;
    let mut instances = Vec::new();
    let mut peripherals = Vec::new();
    let mut interrupts = Vec::new();
    for line in BufReader::new(input).lines() {
        let line = line?;
        if let Some(line) = line.strip_prefix("#define TOP_EARLGREY_") {
            let Some((name, value)) = line.split_once("_BASE_ADDR 0x") else { continue };
            if !IP_BLOCKS.contains(&name.trim_end_matches(|x: char| x.is_ascii_digit())) {
                continue;
            }
            instances.push(Instance {
                name: name.to_string(),
                base: parse_hex(value),
                array: None,
            });
        } else if let Some(line) = line.strip_prefix("  kTopEarlgreyPlicPeripheral") {
            let (name, value) = line.split_once(" = ").unwrap();
            let (value, _) = value.split_once(",").unwrap();
            if !matches!(name, "Unknown" | "Last") {
                peripherals.push(Instance {
                    name: uncamel(name),
                    base: value.parse().unwrap(),
                    array: None,
                });
            }
        } else if let Some(line) = line.strip_prefix("  kTopEarlgreyPlicIrqId") {
            let (name, value) = line.split_once(" = ").unwrap();
            let (value, _) = value.split_once(",").unwrap();
            if !matches!(name, "None" | "Last") {
                assert_eq!(interrupts.len() + 1, value.parse().unwrap());
                interrupts.push(uncamel(name));
            }
        }
    }

    // Compact instances.
    compact(&mut instances);

    // Generate instances.
    for Instance { name, base, array } in &instances {
        let cname = camel(ip_name(name));
        writeln!(
            output,
            "#[rustfmt::skip]\n#[derive(Clone, Copy)]\npub struct r#{cname} {{ pub addr: u32 }}"
        )?;
        write!(output, "#[rustfmt::skip]\npub const r#{name}: ")?;
        match array {
            None => writeln!(output, "r#{cname} = r#{cname} {{ addr: {base:#x} }};")?,
            Some(Array { step, count }) => {
                writeln!(output, "[r#{cname}; {count}] = [")?;
                for i in 0 .. *count {
                    writeln!(output, "r#{cname} {{ addr: {:#x} }},", base + i * step)?;
                }
                writeln!(output, "];")?;
            }
        }
    }

    // Generate interrupts.
    compact(&mut peripherals);
    writeln!(output, "#[rustfmt::skip]\npub mod plic {{")?;
    let mut i = 0;
    for Instance { name, base: _, array } in peripherals {
        let min = i + 1;
        while interrupts.get(i).is_some_and(|x| x.starts_with(&name)) {
            i += 1;
        }
        let max = i;
        if IP_BLOCKS.iter().all(|x| ip_name(x) != name) {
            continue;
        }
        writeln!(output, "pub const r#{name}_MIN: u32 = {min};")?;
        writeln!(output, "pub const r#{name}_MAX: u32 = {max};")?;
        if let Some(Array { step: _, count }) = array {
            writeln!(output, "pub const r#{name}_LEN: u32 = {count};")?;
        }
    }
    writeln!(output, "}}")?;

    // Generate IP blocks.
    for name in IP_BLOCKS {
        let name = ip_name(name);
        let cname = camel(name);
        let lname = name.to_lowercase();
        writeln!(output, "#[rustfmt::skip]\npub mod r#{lname} {{")?;
        let path = format!("{OPENTITAN}/{FASTBUILD}/{}", IpPath::header(&lname));
        let input = File::open(path)?;
        let mut registers = Vec::new();

        // Parse registers.
        let prefix = format!("#define {name}_");
        let mut lines = BufReader::new(input).lines().peekable();
        while let Some(line) = lines.next() {
            let line = line?;
            let Some(line) = line.strip_prefix(&prefix) else { continue };
            let Some((name, offset)) = line.split_once("_REG_OFFSET 0x") else {
                continue;
            };
            let name = name.to_string();
            let offset = parse_hex(offset);
            let prefix = format!("{prefix}{name}_");
            let default = lines.next().unwrap()?;
            let mut fields = Vec::new();
            let Some(default) = default.strip_prefix(&format!("{prefix}REG_RESVAL 0x")) else {
                let count = default.strip_prefix(&format!("{prefix}SIZE_WORDS ")).unwrap();
                let count: u32 = count.parse().unwrap();
                let array = Some(Array { step: 4, count });
                registers.push(Register { name, offset, array, default: 0, fields });
                continue;
            };
            let default = parse_hex(default);
            while let Some(line) = lines.next() {
                let line = line?;
                if line.is_empty() {
                    break;
                }
                let field = line.strip_prefix(&prefix).unwrap();
                if let Some((name, bit)) = field.split_once("_BIT ") {
                    let name = name.to_string();
                    let bit = bit.parse().unwrap();
                    // TODO: Factorize this with Bits.
                    let prefix = format!("{prefix}{name}_VALUE_");
                    while matches!(lines.peek(), Some(Ok(x)) if x.starts_with(&prefix)) {
                        lines.next().unwrap()?;
                    }
                    fields.push(Field::Bit { name, bit, array: None });
                } else if let Some((name, mask)) = field.split_once("_MASK 0x") {
                    let name = name.to_string();
                    let mask = parse_hex(mask);
                    let prefix = format!("{prefix}{name}_");
                    let offset = lines.next().unwrap()?;
                    let offset = offset.strip_prefix(&format!("{prefix}OFFSET ")).unwrap();
                    let offset: u8 = offset.parse().unwrap();
                    let mask = mask << offset;
                    assert!(lines.next().unwrap()?.ends_with("_FIELD \\"));
                    assert!(lines.next().unwrap()?.starts_with("  ((bitfield_field32_t)"));
                    let prefix = format!("{prefix}VALUE_");
                    let mut values = Vec::new();
                    while matches!(lines.peek(), Some(Ok(x)) if x.starts_with(&prefix)) {
                        let value = lines.next().unwrap()?;
                        let value = value.strip_prefix(&prefix).unwrap();
                        let (name, value) = value.split_once(" 0x").unwrap();
                        let name = name.to_string();
                        let value = parse_hex(value);
                        values.push(Value { name, value });
                    }
                    fields.push(Field::Bits { name, mask, values });
                } else if let Some((name, _)) = field.split_once("_VALUE_") {
                    // This happens when the register has a 32 bits field.
                    assert!(fields.is_empty());
                    // TODO: Factorize this with other value iteration.
                    let prefix = format!("{prefix}{name}_VALUE_");
                    while matches!(lines.peek(), Some(Ok(x)) if x.starts_with(&prefix)) {
                        lines.next().unwrap()?;
                    }
                } else {
                    unreachable!();
                }
            }
            registers.push(Register { name, offset, array: None, default, fields });
        }

        // Patch registers.
        for register in &mut registers {
            let Some((n, i)) = split(&register.name) else { continue };
            // TODO: There's a MULTIREG_COUNT suffix. This would fix the ugly hack below.
            let k = match (name, n) {
                ("RV_PLIC", "IP_") => Some("P_"),
                ("RV_PLIC", "IE0_") => Some("E_"),
                ("PINMUX", "MIO_PAD_SLEEP_STATUS_") => Some("EN_"),
                _ => None,
            };
            if let Some(k) = k {
                for field in &mut register.fields {
                    match field {
                        Field::Bit { name, bit, .. } => {
                            let (m, j) = split(name).unwrap();
                            assert_eq!(m, k);
                            assert_eq!(j, 32 * i + *bit);
                            *name = format!("{m}{bit}");
                        }
                        Field::Bits { .. } => unreachable!(),
                    }
                }
                // This is extremely ugly, but enough to make progress.
                while register.fields.len() < 32 {
                    let bit = register.fields.len() as u32;
                    let name = format!("{k}{bit}");
                    register.fields.push(Field::Bit { name, bit, array: None });
                }
            }
            let mut map = HashMap::<String, (u32, u32)>::new();
            for field in &mut register.fields {
                let Some((m, j)) = split(field.name()) else { continue };
                match map.entry(m.to_string()) {
                    Entry::Occupied(mut x) => x.get_mut().1 = j,
                    Entry::Vacant(x) => drop(x.insert((j, j))),
                }
            }
            for field in &mut register.fields {
                let name = field.name_mut();
                let Some((m, j)) = split(name) else { continue };
                let (min, max) = map[m];
                if min == max && min == i {
                    assert_eq!(i, j);
                    *name = split(name).unwrap().0.trim_end_matches('_').to_string();
                }
            }
        }

        // Compact registers.
        compact(&mut registers);
        for register in &mut registers {
            compact(&mut register.fields);
        }

        // Generate IP block implementation.
        writeln!(output, "impl super::r#{cname} {{")?;
        for Register { name, offset, array, default: _, fields: _ } in &registers {
            let cname = camel(name);
            let lname = name.to_lowercase();
            writeln!(output, "#[inline]")?;
            match array {
                Some(Array { step, count }) => {
                    writeln!(
                        output,
                        "pub fn r#{lname}(self, index: u32) -> register::RegAddr<r#{cname}> {{"
                    )?;
                    writeln!(output, "assert!(index < {count});")?;
                    writeln!(
                        output,
                        "unsafe {{ register::RegAddr::new(self.addr + {offset:#x} + index * \
                         {step}) }}"
                    )?;
                }
                None => {
                    writeln!(output, "pub fn r#{lname}(self) -> register::RegAddr<r#{cname}> {{")?;
                    writeln!(
                        output,
                        "unsafe {{ register::RegAddr::new(self.addr + {offset:#x}) }}"
                    )?;
                }
            }
            writeln!(output, "}}")?;
        }
        writeln!(output, "}}")?;

        // Generate registers.
        for Register { name, offset: _, array: _, default, fields } in &registers {
            let cname = camel(name);
            let lname = name.to_lowercase();
            let has_fields = !fields.is_empty();
            let has_enums = fields.iter().any(|x| !x.values().is_empty());

            // Generate specification.
            writeln!(output, "pub enum r#{cname} {{}}")?;
            writeln!(output, "impl register::RegSpec for r#{cname} {{")?;
            writeln!(output, "const DEFAULT: u32 = {default:#x};")?;
            writeln!(output, "type Read = r#{cname}Read;")?;
            writeln!(output, "type Write = r#{cname}Write;")?;
            writeln!(output, "}}")?;

            // Generate read view.
            writeln!(
                output,
                "#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub \
                 struct r#{cname}Read {{ pub reg: register::RegRead<r#{cname}> }}"
            )?;
            if has_fields {
                writeln!(output, "impl r#{cname}Read {{")?;
            }
            for field in fields {
                match field {
                    Field::Bit { name, bit, array } => {
                        let lname = name.to_lowercase();
                        writeln!(output, "#[inline]")?;
                        match array {
                            Some(Array { step, count }) => {
                                writeln!(output, "pub fn r#{lname}(self, index: u8) -> bool {{")?;
                                writeln!(output, "assert!(index < {count});")?;
                                writeln!(output, "self.reg.bit({bit} + index * {step})")?;
                            }
                            None => {
                                writeln!(output, "pub fn r#{lname}(self) -> bool {{")?;
                                writeln!(output, "self.reg.bit({bit})")?;
                            }
                        }
                        writeln!(output, "}}")?;
                    }
                    Field::Bits { name, mask, values: _ } => {
                        let lname = name.to_lowercase();
                        writeln!(output, "#[inline]")?;
                        writeln!(output, "pub fn r#{lname}(self) -> u32 {{")?;
                        writeln!(output, "self.reg.field({mask:#x})")?;
                        writeln!(output, "}}")?;
                    }
                }
            }
            if has_fields {
                writeln!(output, "}}")?;
            }

            // Generate write view.
            writeln!(
                output,
                "#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub \
                 struct r#{cname}Write {{ pub reg: register::RegWrite<r#{cname}> }}"
            )?;
            if has_fields {
                writeln!(output, "impl r#{cname}Write {{")?;
            }
            for field in fields {
                match field {
                    Field::Bit { name, bit, array } => {
                        let lname = name.to_lowercase();
                        writeln!(output, "#[inline]")?;
                        match array {
                            Some(Array { step, count }) => {
                                writeln!(
                                    output,
                                    "pub fn r#{lname}(&mut self, index: u8, value: bool) -> &mut \
                                     Self {{"
                                )?;
                                writeln!(output, "assert!(index < {count});")?;
                                writeln!(
                                    output,
                                    "self.reg.bit({bit} + index * {step}, value); self"
                                )?;
                            }
                            None => {
                                writeln!(
                                    output,
                                    "pub fn r#{lname}(&mut self, value: bool) -> &mut Self {{"
                                )?;
                                writeln!(output, "self.reg.bit({bit}, value); self")?;
                            }
                        }
                        writeln!(output, "}}")?;
                    }
                    Field::Bits { name, mask, values: _ } => {
                        let lname = name.to_lowercase();
                        writeln!(output, "#[inline]")?;
                        writeln!(
                            output,
                            "pub fn r#{lname}(&mut self, value: u32) -> &mut Self {{"
                        )?;
                        writeln!(output, "self.reg.field({mask:#x}, value); self")?;
                        writeln!(output, "}}")?;
                    }
                }
            }
            if has_fields {
                writeln!(output, "}}")?;
            }

            // Generate enum values.
            if !has_enums {
                continue;
            }
            writeln!(output, "pub mod r#{lname} {{")?;
            for field in fields {
                let (name, values) = match field {
                    Field::Bits { name, values, .. } if !values.is_empty() => (name, values),
                    _ => continue,
                };
                let cname = camel(name);
                writeln!(output, "#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]")?;
                writeln!(output, "#[repr(u32)]")?;
                writeln!(output, "pub enum r#{cname} {{")?;
                for Value { name, value } in values {
                    let cname = camel(name);
                    writeln!(output, "r#{cname} = {value:#x},")?;
                }
                writeln!(output, "}}")?;
            }
            writeln!(output, "}}")?;
        }
        writeln!(output, "}}")?;
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Array {
    step: u32,
    count: u32,
}

#[derive(Debug)]
struct Instance {
    name: String,
    base: u32,
    array: Option<Array>,
}

#[derive(Debug)]
struct Register {
    name: String,
    offset: u32,
    array: Option<Array>,
    default: u32,
    fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Eq)]
enum Field {
    Bit { name: String, bit: u32, array: Option<Array> },
    Bits { name: String, mask: u32, values: Vec<Value> },
}

#[derive(Debug, PartialEq, Eq)]
struct Value {
    name: String,
    value: u32,
}

impl Field {
    fn name(&self) -> &str {
        match self {
            Field::Bit { name, .. } | Field::Bits { name, .. } => name,
        }
    }

    fn name_mut(&mut self) -> &mut String {
        match self {
            Field::Bit { name, .. } | Field::Bits { name, .. } => name,
        }
    }

    fn values(&self) -> &[Value] {
        match self {
            Field::Bit { .. } => &[],
            Field::Bits { values, .. } => values,
        }
    }
}

trait Compact: Debug {
    fn en_name(&mut self) -> Option<&mut String>;
    fn value(&self) -> u32;
    fn array(&mut self) -> &mut Option<Array>;
    fn matches(&self, other: &Self) -> bool;
}

impl Compact for Instance {
    fn en_name(&mut self) -> Option<&mut String> {
        Some(&mut self.name)
    }
    fn value(&self) -> u32 {
        self.base
    }
    fn array(&mut self) -> &mut Option<Array> {
        &mut self.array
    }
    fn matches(&self, _: &Self) -> bool {
        true
    }
}

impl Compact for Register {
    fn en_name(&mut self) -> Option<&mut String> {
        Some(&mut self.name)
    }
    fn value(&self) -> u32 {
        self.offset
    }
    fn array(&mut self) -> &mut Option<Array> {
        &mut self.array
    }
    fn matches(&self, other: &Self) -> bool {
        if self.default != other.default || self.fields.len() != other.fields.len() {
            return false;
        }
        for (x, y) in std::iter::zip(&self.fields, &other.fields) {
            let matches = match (x, y) {
                (Field::Bit { name: xn, bit: xb, .. }, Field::Bit { name: yn, bit: yb, .. }) => {
                    xn == yn && xb == yb
                }
                (
                    Field::Bits { name: xn, mask: xm, .. },
                    Field::Bits { name: yn, mask: ym, .. },
                ) => xn == yn && xm == ym,
                _ => false,
            };
            if !matches {
                return false;
            }
        }
        true
    }
}

impl Compact for Field {
    fn en_name(&mut self) -> Option<&mut String> {
        match self {
            Field::Bit { name, .. } => Some(name),
            Field::Bits { .. } => None,
        }
    }
    fn value(&self) -> u32 {
        match self {
            Field::Bit { bit, .. } => *bit,
            Field::Bits { .. } => unreachable!(),
        }
    }
    fn array(&mut self) -> &mut Option<Array> {
        match self {
            Field::Bit { array, .. } => array,
            Field::Bits { .. } => unreachable!(),
        }
    }
    fn matches(&self, _: &Self) -> bool {
        true
    }
}

/// Compacts consecutive elements with linear progression.
fn compact<T: Compact>(xs: &mut Vec<T>) {
    let mut ys = Vec::new();
    for mut x in std::mem::take(xs) {
        if x.en_name().is_none() || x.array().is_some() {
            ys.push(x);
            continue;
        }
        let Some((n, i)) = split(x.en_name().unwrap()) else {
            ys.push(x);
            continue;
        };
        // Corner cases.
        if matches!((n, i), ("CRC", 5 | 16) | ("HW_REVISION", _)) {
            ys.push(x);
            continue;
        }
        let n = n.trim_end_matches('_').to_string();
        if i == 0 {
            *x.en_name().unwrap() = n;
            *x.array() = Some(Array { step: 0, count: 1 });
            ys.push(x);
            continue;
        }
        let y = ys.last_mut().unwrap();
        assert!(x.matches(y));
        assert_eq!(*y.en_name().unwrap(), n);
        let d = x.value() - y.value();
        let a = y.array().as_mut().unwrap();
        if a.count == 1 {
            a.step = d;
        }
        assert!(1 <= a.count);
        assert_eq!(d, i * a.step);
        a.count += 1;
    }
    *xs = ys;
}

/// Split `FOO42` into `(FOO, 42)`.
fn split(n: &str) -> Option<(&str, u32)> {
    let p = n.rfind(|x: char| !x.is_ascii_digit()).unwrap() + 1;
    if p == n.len() {
        return None;
    }
    let i = n[p ..].parse().unwrap();
    Some((&n[.. p], i))
}

/// Converts `HELLO_WORLD` to `HelloWorld`.
fn camel(name: &str) -> String {
    let mut result = String::new();
    let mut underscore = true;
    for c in name.chars() {
        if c == '_' {
            underscore = true;
        } else if underscore {
            underscore = false;
            result.push(c);
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

/// Converts `HelloWorld` to `HELLO_WORLD`.
fn uncamel(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() && !result.is_empty() {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}

/// Parses an hexadecimal number (possibly suffixed by "u").
fn parse_hex(input: &str) -> u32 {
    u32::from_str_radix(input.strip_suffix("u").unwrap_or(input), 16).unwrap()
}

fn ip_name(name: &str) -> &str {
    match name {
        "FLASH_CTRL_CORE" => "FLASH_CTRL",
        "PINMUX_AON" => "PINMUX",
        "RSTMGR_AON" => "RSTMGR",
        x => x,
    }
}

#[derive(Clone, Copy)]
enum IpPath {
    TopAuto,
    Top,
    Ip,
}

impl IpPath {
    fn new(lname: &str) -> Self {
        for (prefix, result) in [
            ("hw/top_earlgrey/ip_autogen", IpPath::TopAuto),
            ("hw/top_earlgrey/ip", IpPath::Top),
            ("hw/ip", IpPath::Ip),
        ] {
            if std::fs::exists(format!("{OPENTITAN}/{prefix}/{lname}")).is_ok_and(|x| x) {
                return result;
            }
        }
        unreachable!("{lname}")
    }

    fn build(lname: &str) -> String {
        let mut result = match Self::new(lname) {
            _ if lname == "rv_plic" => "//hw/top_earlgrey".to_string(),
            IpPath::TopAuto => format!("//hw/top_earlgrey/ip_autogen/{lname}"),
            IpPath::Top => format!("//hw/top_earlgrey/ip/{lname}/data/autogen"),
            IpPath::Ip => format!("//hw/ip/{lname}/data"),
        };
        write!(result, ":{lname}_c_regs").unwrap();
        result
    }

    fn header(lname: &str) -> String {
        let mut result = match Self::new(lname) {
            _ if lname == "rv_plic" => "hw/top_earlgrey".to_string(),
            IpPath::TopAuto => format!("hw/top_earlgrey/ip_autogen/{lname}"),
            IpPath::Top => format!("hw/top_earlgrey/ip/{lname}/data/autogen"),
            IpPath::Ip => format!("hw/ip/{lname}/data"),
        };
        write!(result, "/{lname}_regs.h").unwrap();
        result
    }
}
