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

// This file is generated from crates/generate.

#![no_std]
#![allow(clippy::erasing_op)]
#![allow(clippy::identity_op)]
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#Uart { pub addr: u32 }
#[rustfmt::skip]
pub const r#UART: [r#Uart; 4] = [
r#Uart { addr: 0x40000000 },
r#Uart { addr: 0x40010000 },
r#Uart { addr: 0x40020000 },
r#Uart { addr: 0x40030000 },
];
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#RvTimer { pub addr: u32 }
#[rustfmt::skip]
pub const r#RV_TIMER: r#RvTimer = r#RvTimer { addr: 0x40100000 };
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#LcCtrl { pub addr: u32 }
#[rustfmt::skip]
pub const r#LC_CTRL: r#LcCtrl = r#LcCtrl { addr: 0x40140000 };
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#Usbdev { pub addr: u32 }
#[rustfmt::skip]
pub const r#USBDEV: r#Usbdev = r#Usbdev { addr: 0x40320000 };
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#Rstmgr { pub addr: u32 }
#[rustfmt::skip]
pub const r#RSTMGR_AON: r#Rstmgr = r#Rstmgr { addr: 0x40410000 };
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#Pinmux { pub addr: u32 }
#[rustfmt::skip]
pub const r#PINMUX_AON: r#Pinmux = r#Pinmux { addr: 0x40460000 };
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#FlashCtrl { pub addr: u32 }
#[rustfmt::skip]
pub const r#FLASH_CTRL_CORE: r#FlashCtrl = r#FlashCtrl { addr: 0x41000000 };
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#RvPlic { pub addr: u32 }
#[rustfmt::skip]
pub const r#RV_PLIC: r#RvPlic = r#RvPlic { addr: 0x48000000 };
#[rustfmt::skip]
#[derive(Clone, Copy)]
pub struct r#Hmac { pub addr: u32 }
#[rustfmt::skip]
pub const r#HMAC: r#Hmac = r#Hmac { addr: 0x41110000 };
#[rustfmt::skip]
pub mod plic {
pub const r#UART_MIN: u32 = 1;
pub const r#UART_MAX: u32 = 32;
pub const r#UART_LEN: u32 = 4;
pub const r#RV_TIMER_MIN: u32 = 124;
pub const r#RV_TIMER_MAX: u32 = 124;
pub const r#USBDEV_MIN: u32 = 135;
pub const r#USBDEV_MAX: u32 = 151;
pub const r#FLASH_CTRL_MIN: u32 = 159;
pub const r#FLASH_CTRL_MAX: u32 = 164;
pub const r#HMAC_MIN: u32 = 165;
pub const r#HMAC_MAX: u32 = 167;
}
#[rustfmt::skip]
pub mod r#flash_ctrl {
impl super::r#FlashCtrl {
#[inline]
pub fn r#intr_state(self) -> register::RegAddr<r#IntrState> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#intr_enable(self) -> register::RegAddr<r#IntrEnable> {
unsafe { register::RegAddr::new(self.addr + 0x4) }
}
#[inline]
pub fn r#intr_test(self) -> register::RegAddr<r#IntrTest> {
unsafe { register::RegAddr::new(self.addr + 0x8) }
}
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0xc) }
}
#[inline]
pub fn r#dis(self) -> register::RegAddr<r#Dis> {
unsafe { register::RegAddr::new(self.addr + 0x10) }
}
#[inline]
pub fn r#exec(self) -> register::RegAddr<r#Exec> {
unsafe { register::RegAddr::new(self.addr + 0x14) }
}
#[inline]
pub fn r#init(self) -> register::RegAddr<r#Init> {
unsafe { register::RegAddr::new(self.addr + 0x18) }
}
#[inline]
pub fn r#ctrl_regwen(self) -> register::RegAddr<r#CtrlRegwen> {
unsafe { register::RegAddr::new(self.addr + 0x1c) }
}
#[inline]
pub fn r#control(self) -> register::RegAddr<r#Control> {
unsafe { register::RegAddr::new(self.addr + 0x20) }
}
#[inline]
pub fn r#addr(self) -> register::RegAddr<r#Addr> {
unsafe { register::RegAddr::new(self.addr + 0x24) }
}
#[inline]
pub fn r#prog_type_en(self) -> register::RegAddr<r#ProgTypeEn> {
unsafe { register::RegAddr::new(self.addr + 0x28) }
}
#[inline]
pub fn r#erase_suspend(self) -> register::RegAddr<r#EraseSuspend> {
unsafe { register::RegAddr::new(self.addr + 0x2c) }
}
#[inline]
pub fn r#region_cfg_regwen(self, index: u32) -> register::RegAddr<r#RegionCfgRegwen> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x30 + index * 4) }
}
#[inline]
pub fn r#mp_region_cfg(self, index: u32) -> register::RegAddr<r#MpRegionCfg> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x50 + index * 4) }
}
#[inline]
pub fn r#mp_region(self, index: u32) -> register::RegAddr<r#MpRegion> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x70 + index * 4) }
}
#[inline]
pub fn r#default_region(self) -> register::RegAddr<r#DefaultRegion> {
unsafe { register::RegAddr::new(self.addr + 0x90) }
}
#[inline]
pub fn r#bank0_info0_regwen(self, index: u32) -> register::RegAddr<r#Bank0Info0Regwen> {
assert!(index < 10);
unsafe { register::RegAddr::new(self.addr + 0x94 + index * 4) }
}
#[inline]
pub fn r#bank0_info0_page_cfg(self, index: u32) -> register::RegAddr<r#Bank0Info0PageCfg> {
assert!(index < 10);
unsafe { register::RegAddr::new(self.addr + 0xbc + index * 4) }
}
#[inline]
pub fn r#bank0_info1_regwen(self) -> register::RegAddr<r#Bank0Info1Regwen> {
unsafe { register::RegAddr::new(self.addr + 0xe4) }
}
#[inline]
pub fn r#bank0_info1_page_cfg(self) -> register::RegAddr<r#Bank0Info1PageCfg> {
unsafe { register::RegAddr::new(self.addr + 0xe8) }
}
#[inline]
pub fn r#bank0_info2_regwen(self, index: u32) -> register::RegAddr<r#Bank0Info2Regwen> {
assert!(index < 2);
unsafe { register::RegAddr::new(self.addr + 0xec + index * 4) }
}
#[inline]
pub fn r#bank0_info2_page_cfg(self, index: u32) -> register::RegAddr<r#Bank0Info2PageCfg> {
assert!(index < 2);
unsafe { register::RegAddr::new(self.addr + 0xf4 + index * 4) }
}
#[inline]
pub fn r#bank1_info0_regwen(self, index: u32) -> register::RegAddr<r#Bank1Info0Regwen> {
assert!(index < 10);
unsafe { register::RegAddr::new(self.addr + 0xfc + index * 4) }
}
#[inline]
pub fn r#bank1_info0_page_cfg(self, index: u32) -> register::RegAddr<r#Bank1Info0PageCfg> {
assert!(index < 10);
unsafe { register::RegAddr::new(self.addr + 0x124 + index * 4) }
}
#[inline]
pub fn r#bank1_info1_regwen(self) -> register::RegAddr<r#Bank1Info1Regwen> {
unsafe { register::RegAddr::new(self.addr + 0x14c) }
}
#[inline]
pub fn r#bank1_info1_page_cfg(self) -> register::RegAddr<r#Bank1Info1PageCfg> {
unsafe { register::RegAddr::new(self.addr + 0x150) }
}
#[inline]
pub fn r#bank1_info2_regwen(self, index: u32) -> register::RegAddr<r#Bank1Info2Regwen> {
assert!(index < 2);
unsafe { register::RegAddr::new(self.addr + 0x154 + index * 4) }
}
#[inline]
pub fn r#bank1_info2_page_cfg(self, index: u32) -> register::RegAddr<r#Bank1Info2PageCfg> {
assert!(index < 2);
unsafe { register::RegAddr::new(self.addr + 0x15c + index * 4) }
}
#[inline]
pub fn r#hw_info_cfg_override(self) -> register::RegAddr<r#HwInfoCfgOverride> {
unsafe { register::RegAddr::new(self.addr + 0x164) }
}
#[inline]
pub fn r#bank_cfg_regwen(self) -> register::RegAddr<r#BankCfgRegwen> {
unsafe { register::RegAddr::new(self.addr + 0x168) }
}
#[inline]
pub fn r#mp_bank_cfg_shadowed(self) -> register::RegAddr<r#MpBankCfgShadowed> {
unsafe { register::RegAddr::new(self.addr + 0x16c) }
}
#[inline]
pub fn r#op_status(self) -> register::RegAddr<r#OpStatus> {
unsafe { register::RegAddr::new(self.addr + 0x170) }
}
#[inline]
pub fn r#status(self) -> register::RegAddr<r#Status> {
unsafe { register::RegAddr::new(self.addr + 0x174) }
}
#[inline]
pub fn r#debug_state(self) -> register::RegAddr<r#DebugState> {
unsafe { register::RegAddr::new(self.addr + 0x178) }
}
#[inline]
pub fn r#err_code(self) -> register::RegAddr<r#ErrCode> {
unsafe { register::RegAddr::new(self.addr + 0x17c) }
}
#[inline]
pub fn r#std_fault_status(self) -> register::RegAddr<r#StdFaultStatus> {
unsafe { register::RegAddr::new(self.addr + 0x180) }
}
#[inline]
pub fn r#fault_status(self) -> register::RegAddr<r#FaultStatus> {
unsafe { register::RegAddr::new(self.addr + 0x184) }
}
#[inline]
pub fn r#err_addr(self) -> register::RegAddr<r#ErrAddr> {
unsafe { register::RegAddr::new(self.addr + 0x188) }
}
#[inline]
pub fn r#ecc_single_err_cnt(self) -> register::RegAddr<r#EccSingleErrCnt> {
unsafe { register::RegAddr::new(self.addr + 0x18c) }
}
#[inline]
pub fn r#ecc_single_err_addr(self, index: u32) -> register::RegAddr<r#EccSingleErrAddr> {
assert!(index < 2);
unsafe { register::RegAddr::new(self.addr + 0x190 + index * 4) }
}
#[inline]
pub fn r#phy_alert_cfg(self) -> register::RegAddr<r#PhyAlertCfg> {
unsafe { register::RegAddr::new(self.addr + 0x198) }
}
#[inline]
pub fn r#phy_status(self) -> register::RegAddr<r#PhyStatus> {
unsafe { register::RegAddr::new(self.addr + 0x19c) }
}
#[inline]
pub fn r#scratch(self) -> register::RegAddr<r#Scratch> {
unsafe { register::RegAddr::new(self.addr + 0x1a0) }
}
#[inline]
pub fn r#fifo_lvl(self) -> register::RegAddr<r#FifoLvl> {
unsafe { register::RegAddr::new(self.addr + 0x1a4) }
}
#[inline]
pub fn r#fifo_rst(self) -> register::RegAddr<r#FifoRst> {
unsafe { register::RegAddr::new(self.addr + 0x1a8) }
}
#[inline]
pub fn r#curr_fifo_lvl(self) -> register::RegAddr<r#CurrFifoLvl> {
unsafe { register::RegAddr::new(self.addr + 0x1ac) }
}
#[inline]
pub fn r#prog_fifo(self, index: u32) -> register::RegAddr<r#ProgFifo> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x1b0 + index * 4) }
}
#[inline]
pub fn r#rd_fifo(self, index: u32) -> register::RegAddr<r#RdFifo> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x1b4 + index * 4) }
}
}
pub enum r#IntrState {}
impl register::RegSpec for r#IntrState {
const DEFAULT: u32 = 0x0;
type Read = r#IntrStateRead;
type Write = r#IntrStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateRead { pub reg: register::RegRead<r#IntrState> }
impl r#IntrStateRead {
#[inline]
pub fn r#prog_empty(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#prog_lvl(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#rd_full(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#rd_lvl(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#op_done(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#corr_err(self) -> bool {
self.reg.bit(5)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateWrite { pub reg: register::RegWrite<r#IntrState> }
impl r#IntrStateWrite {
#[inline]
pub fn r#prog_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#prog_lvl(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#rd_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#rd_lvl(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#op_done(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#corr_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
}
pub enum r#IntrEnable {}
impl register::RegSpec for r#IntrEnable {
const DEFAULT: u32 = 0x0;
type Read = r#IntrEnableRead;
type Write = r#IntrEnableWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableRead { pub reg: register::RegRead<r#IntrEnable> }
impl r#IntrEnableRead {
#[inline]
pub fn r#prog_empty(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#prog_lvl(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#rd_full(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#rd_lvl(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#op_done(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#corr_err(self) -> bool {
self.reg.bit(5)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableWrite { pub reg: register::RegWrite<r#IntrEnable> }
impl r#IntrEnableWrite {
#[inline]
pub fn r#prog_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#prog_lvl(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#rd_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#rd_lvl(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#op_done(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#corr_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
}
pub enum r#IntrTest {}
impl register::RegSpec for r#IntrTest {
const DEFAULT: u32 = 0x0;
type Read = r#IntrTestRead;
type Write = r#IntrTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestRead { pub reg: register::RegRead<r#IntrTest> }
impl r#IntrTestRead {
#[inline]
pub fn r#prog_empty(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#prog_lvl(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#rd_full(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#rd_lvl(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#op_done(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#corr_err(self) -> bool {
self.reg.bit(5)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestWrite { pub reg: register::RegWrite<r#IntrTest> }
impl r#IntrTestWrite {
#[inline]
pub fn r#prog_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#prog_lvl(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#rd_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#rd_lvl(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#op_done(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#corr_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#recov_err(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#fatal_std_err(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#fatal_err(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#fatal_prim_flash_alert(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#recov_prim_flash_alert(self) -> bool {
self.reg.bit(4)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#recov_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#fatal_std_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#fatal_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#fatal_prim_flash_alert(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#recov_prim_flash_alert(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
}
pub enum r#Dis {}
impl register::RegSpec for r#Dis {
const DEFAULT: u32 = 0x9;
type Read = r#DisRead;
type Write = r#DisWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DisRead { pub reg: register::RegRead<r#Dis> }
impl r#DisRead {
#[inline]
pub fn r#val(self) -> u32 {
self.reg.field(0xf)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DisWrite { pub reg: register::RegWrite<r#Dis> }
impl r#DisWrite {
#[inline]
pub fn r#val(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
}
pub enum r#Exec {}
impl register::RegSpec for r#Exec {
const DEFAULT: u32 = 0x0;
type Read = r#ExecRead;
type Write = r#ExecWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ExecRead { pub reg: register::RegRead<r#Exec> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ExecWrite { pub reg: register::RegWrite<r#Exec> }
pub enum r#Init {}
impl register::RegSpec for r#Init {
const DEFAULT: u32 = 0x0;
type Read = r#InitRead;
type Write = r#InitWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InitRead { pub reg: register::RegRead<r#Init> }
impl r#InitRead {
#[inline]
pub fn r#val(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InitWrite { pub reg: register::RegWrite<r#Init> }
impl r#InitWrite {
#[inline]
pub fn r#val(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#CtrlRegwen {}
impl register::RegSpec for r#CtrlRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#CtrlRegwenRead;
type Write = r#CtrlRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CtrlRegwenRead { pub reg: register::RegRead<r#CtrlRegwen> }
impl r#CtrlRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CtrlRegwenWrite { pub reg: register::RegWrite<r#CtrlRegwen> }
impl r#CtrlRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Control {}
impl register::RegSpec for r#Control {
const DEFAULT: u32 = 0x0;
type Read = r#ControlRead;
type Write = r#ControlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ControlRead { pub reg: register::RegRead<r#Control> }
impl r#ControlRead {
#[inline]
pub fn r#start(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#op(self) -> u32 {
self.reg.field(0x30)
}
#[inline]
pub fn r#prog_sel(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#erase_sel(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#partition_sel(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#info_sel(self) -> u32 {
self.reg.field(0x600)
}
#[inline]
pub fn r#num(self) -> u32 {
self.reg.field(0xfff0000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ControlWrite { pub reg: register::RegWrite<r#Control> }
impl r#ControlWrite {
#[inline]
pub fn r#start(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#op(&mut self, value: u32) -> &mut Self {
self.reg.field(0x30, value); self
}
#[inline]
pub fn r#prog_sel(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#erase_sel(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#partition_sel(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#info_sel(&mut self, value: u32) -> &mut Self {
self.reg.field(0x600, value); self
}
#[inline]
pub fn r#num(&mut self, value: u32) -> &mut Self {
self.reg.field(0xfff0000, value); self
}
}
pub mod r#control {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#Op {
r#Read = 0x0,
r#Prog = 0x1,
r#Erase = 0x2,
}
}
pub enum r#Addr {}
impl register::RegSpec for r#Addr {
const DEFAULT: u32 = 0x0;
type Read = r#AddrRead;
type Write = r#AddrWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AddrRead { pub reg: register::RegRead<r#Addr> }
impl r#AddrRead {
#[inline]
pub fn r#start(self) -> u32 {
self.reg.field(0xfffff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AddrWrite { pub reg: register::RegWrite<r#Addr> }
impl r#AddrWrite {
#[inline]
pub fn r#start(&mut self, value: u32) -> &mut Self {
self.reg.field(0xfffff, value); self
}
}
pub enum r#ProgTypeEn {}
impl register::RegSpec for r#ProgTypeEn {
const DEFAULT: u32 = 0x3;
type Read = r#ProgTypeEnRead;
type Write = r#ProgTypeEnWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ProgTypeEnRead { pub reg: register::RegRead<r#ProgTypeEn> }
impl r#ProgTypeEnRead {
#[inline]
pub fn r#normal(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#repair(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ProgTypeEnWrite { pub reg: register::RegWrite<r#ProgTypeEn> }
impl r#ProgTypeEnWrite {
#[inline]
pub fn r#normal(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#repair(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#EraseSuspend {}
impl register::RegSpec for r#EraseSuspend {
const DEFAULT: u32 = 0x0;
type Read = r#EraseSuspendRead;
type Write = r#EraseSuspendWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EraseSuspendRead { pub reg: register::RegRead<r#EraseSuspend> }
impl r#EraseSuspendRead {
#[inline]
pub fn r#req(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EraseSuspendWrite { pub reg: register::RegWrite<r#EraseSuspend> }
impl r#EraseSuspendWrite {
#[inline]
pub fn r#req(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#RegionCfgRegwen {}
impl register::RegSpec for r#RegionCfgRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#RegionCfgRegwenRead;
type Write = r#RegionCfgRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RegionCfgRegwenRead { pub reg: register::RegRead<r#RegionCfgRegwen> }
impl r#RegionCfgRegwenRead {
#[inline]
pub fn r#region(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RegionCfgRegwenWrite { pub reg: register::RegWrite<r#RegionCfgRegwen> }
impl r#RegionCfgRegwenWrite {
#[inline]
pub fn r#region(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MpRegionCfg {}
impl register::RegSpec for r#MpRegionCfg {
const DEFAULT: u32 = 0x9999999;
type Read = r#MpRegionCfgRead;
type Write = r#MpRegionCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MpRegionCfgRead { pub reg: register::RegRead<r#MpRegionCfg> }
impl r#MpRegionCfgRead {
#[inline]
pub fn r#en(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#rd_en(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#prog_en(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#erase_en(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#scramble_en(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#ecc_en(self) -> u32 {
self.reg.field(0xf00000)
}
#[inline]
pub fn r#he_en(self) -> u32 {
self.reg.field(0xf000000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MpRegionCfgWrite { pub reg: register::RegWrite<r#MpRegionCfg> }
impl r#MpRegionCfgWrite {
#[inline]
pub fn r#en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#rd_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#prog_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#erase_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#scramble_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#ecc_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
#[inline]
pub fn r#he_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
}
pub enum r#MpRegion {}
impl register::RegSpec for r#MpRegion {
const DEFAULT: u32 = 0x0;
type Read = r#MpRegionRead;
type Write = r#MpRegionWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MpRegionRead { pub reg: register::RegRead<r#MpRegion> }
impl r#MpRegionRead {
#[inline]
pub fn r#base(self) -> u32 {
self.reg.field(0x1ff)
}
#[inline]
pub fn r#size(self) -> u32 {
self.reg.field(0x7fe00)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MpRegionWrite { pub reg: register::RegWrite<r#MpRegion> }
impl r#MpRegionWrite {
#[inline]
pub fn r#base(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1ff, value); self
}
#[inline]
pub fn r#size(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7fe00, value); self
}
}
pub enum r#DefaultRegion {}
impl register::RegSpec for r#DefaultRegion {
const DEFAULT: u32 = 0x999999;
type Read = r#DefaultRegionRead;
type Write = r#DefaultRegionWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DefaultRegionRead { pub reg: register::RegRead<r#DefaultRegion> }
impl r#DefaultRegionRead {
#[inline]
pub fn r#rd_en(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#prog_en(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#erase_en(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#scramble_en(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#ecc_en(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#he_en(self) -> u32 {
self.reg.field(0xf00000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DefaultRegionWrite { pub reg: register::RegWrite<r#DefaultRegion> }
impl r#DefaultRegionWrite {
#[inline]
pub fn r#rd_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#prog_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#erase_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#scramble_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#ecc_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#he_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
}
pub enum r#Bank0Info0Regwen {}
impl register::RegSpec for r#Bank0Info0Regwen {
const DEFAULT: u32 = 0x1;
type Read = r#Bank0Info0RegwenRead;
type Write = r#Bank0Info0RegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info0RegwenRead { pub reg: register::RegRead<r#Bank0Info0Regwen> }
impl r#Bank0Info0RegwenRead {
#[inline]
pub fn r#region(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info0RegwenWrite { pub reg: register::RegWrite<r#Bank0Info0Regwen> }
impl r#Bank0Info0RegwenWrite {
#[inline]
pub fn r#region(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Bank0Info0PageCfg {}
impl register::RegSpec for r#Bank0Info0PageCfg {
const DEFAULT: u32 = 0x9999999;
type Read = r#Bank0Info0PageCfgRead;
type Write = r#Bank0Info0PageCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info0PageCfgRead { pub reg: register::RegRead<r#Bank0Info0PageCfg> }
impl r#Bank0Info0PageCfgRead {
#[inline]
pub fn r#en(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#rd_en(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#prog_en(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#erase_en(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#scramble_en(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#ecc_en(self) -> u32 {
self.reg.field(0xf00000)
}
#[inline]
pub fn r#he_en(self) -> u32 {
self.reg.field(0xf000000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info0PageCfgWrite { pub reg: register::RegWrite<r#Bank0Info0PageCfg> }
impl r#Bank0Info0PageCfgWrite {
#[inline]
pub fn r#en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#rd_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#prog_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#erase_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#scramble_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#ecc_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
#[inline]
pub fn r#he_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
}
pub enum r#Bank0Info1Regwen {}
impl register::RegSpec for r#Bank0Info1Regwen {
const DEFAULT: u32 = 0x1;
type Read = r#Bank0Info1RegwenRead;
type Write = r#Bank0Info1RegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info1RegwenRead { pub reg: register::RegRead<r#Bank0Info1Regwen> }
impl r#Bank0Info1RegwenRead {
#[inline]
pub fn r#region(self, index: u8) -> bool {
assert!(index < 1);
self.reg.bit(0 + index * 0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info1RegwenWrite { pub reg: register::RegWrite<r#Bank0Info1Regwen> }
impl r#Bank0Info1RegwenWrite {
#[inline]
pub fn r#region(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 1);
self.reg.bit(0 + index * 0, value); self
}
}
pub enum r#Bank0Info1PageCfg {}
impl register::RegSpec for r#Bank0Info1PageCfg {
const DEFAULT: u32 = 0x9999999;
type Read = r#Bank0Info1PageCfgRead;
type Write = r#Bank0Info1PageCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info1PageCfgRead { pub reg: register::RegRead<r#Bank0Info1PageCfg> }
impl r#Bank0Info1PageCfgRead {
#[inline]
pub fn r#en_0(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#rd_en_0(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#prog_en_0(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#erase_en_0(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#scramble_en_0(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#ecc_en_0(self) -> u32 {
self.reg.field(0xf00000)
}
#[inline]
pub fn r#he_en_0(self) -> u32 {
self.reg.field(0xf000000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info1PageCfgWrite { pub reg: register::RegWrite<r#Bank0Info1PageCfg> }
impl r#Bank0Info1PageCfgWrite {
#[inline]
pub fn r#en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#rd_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#prog_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#erase_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#scramble_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#ecc_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
#[inline]
pub fn r#he_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
}
pub enum r#Bank0Info2Regwen {}
impl register::RegSpec for r#Bank0Info2Regwen {
const DEFAULT: u32 = 0x1;
type Read = r#Bank0Info2RegwenRead;
type Write = r#Bank0Info2RegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info2RegwenRead { pub reg: register::RegRead<r#Bank0Info2Regwen> }
impl r#Bank0Info2RegwenRead {
#[inline]
pub fn r#region(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info2RegwenWrite { pub reg: register::RegWrite<r#Bank0Info2Regwen> }
impl r#Bank0Info2RegwenWrite {
#[inline]
pub fn r#region(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Bank0Info2PageCfg {}
impl register::RegSpec for r#Bank0Info2PageCfg {
const DEFAULT: u32 = 0x9999999;
type Read = r#Bank0Info2PageCfgRead;
type Write = r#Bank0Info2PageCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info2PageCfgRead { pub reg: register::RegRead<r#Bank0Info2PageCfg> }
impl r#Bank0Info2PageCfgRead {
#[inline]
pub fn r#en(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#rd_en(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#prog_en(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#erase_en(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#scramble_en(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#ecc_en(self) -> u32 {
self.reg.field(0xf00000)
}
#[inline]
pub fn r#he_en(self) -> u32 {
self.reg.field(0xf000000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank0Info2PageCfgWrite { pub reg: register::RegWrite<r#Bank0Info2PageCfg> }
impl r#Bank0Info2PageCfgWrite {
#[inline]
pub fn r#en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#rd_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#prog_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#erase_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#scramble_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#ecc_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
#[inline]
pub fn r#he_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
}
pub enum r#Bank1Info0Regwen {}
impl register::RegSpec for r#Bank1Info0Regwen {
const DEFAULT: u32 = 0x1;
type Read = r#Bank1Info0RegwenRead;
type Write = r#Bank1Info0RegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info0RegwenRead { pub reg: register::RegRead<r#Bank1Info0Regwen> }
impl r#Bank1Info0RegwenRead {
#[inline]
pub fn r#region(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info0RegwenWrite { pub reg: register::RegWrite<r#Bank1Info0Regwen> }
impl r#Bank1Info0RegwenWrite {
#[inline]
pub fn r#region(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Bank1Info0PageCfg {}
impl register::RegSpec for r#Bank1Info0PageCfg {
const DEFAULT: u32 = 0x9999999;
type Read = r#Bank1Info0PageCfgRead;
type Write = r#Bank1Info0PageCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info0PageCfgRead { pub reg: register::RegRead<r#Bank1Info0PageCfg> }
impl r#Bank1Info0PageCfgRead {
#[inline]
pub fn r#en(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#rd_en(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#prog_en(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#erase_en(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#scramble_en(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#ecc_en(self) -> u32 {
self.reg.field(0xf00000)
}
#[inline]
pub fn r#he_en(self) -> u32 {
self.reg.field(0xf000000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info0PageCfgWrite { pub reg: register::RegWrite<r#Bank1Info0PageCfg> }
impl r#Bank1Info0PageCfgWrite {
#[inline]
pub fn r#en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#rd_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#prog_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#erase_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#scramble_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#ecc_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
#[inline]
pub fn r#he_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
}
pub enum r#Bank1Info1Regwen {}
impl register::RegSpec for r#Bank1Info1Regwen {
const DEFAULT: u32 = 0x1;
type Read = r#Bank1Info1RegwenRead;
type Write = r#Bank1Info1RegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info1RegwenRead { pub reg: register::RegRead<r#Bank1Info1Regwen> }
impl r#Bank1Info1RegwenRead {
#[inline]
pub fn r#region(self, index: u8) -> bool {
assert!(index < 1);
self.reg.bit(0 + index * 0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info1RegwenWrite { pub reg: register::RegWrite<r#Bank1Info1Regwen> }
impl r#Bank1Info1RegwenWrite {
#[inline]
pub fn r#region(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 1);
self.reg.bit(0 + index * 0, value); self
}
}
pub enum r#Bank1Info1PageCfg {}
impl register::RegSpec for r#Bank1Info1PageCfg {
const DEFAULT: u32 = 0x9999999;
type Read = r#Bank1Info1PageCfgRead;
type Write = r#Bank1Info1PageCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info1PageCfgRead { pub reg: register::RegRead<r#Bank1Info1PageCfg> }
impl r#Bank1Info1PageCfgRead {
#[inline]
pub fn r#en_0(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#rd_en_0(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#prog_en_0(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#erase_en_0(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#scramble_en_0(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#ecc_en_0(self) -> u32 {
self.reg.field(0xf00000)
}
#[inline]
pub fn r#he_en_0(self) -> u32 {
self.reg.field(0xf000000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info1PageCfgWrite { pub reg: register::RegWrite<r#Bank1Info1PageCfg> }
impl r#Bank1Info1PageCfgWrite {
#[inline]
pub fn r#en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#rd_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#prog_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#erase_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#scramble_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#ecc_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
#[inline]
pub fn r#he_en_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
}
pub enum r#Bank1Info2Regwen {}
impl register::RegSpec for r#Bank1Info2Regwen {
const DEFAULT: u32 = 0x1;
type Read = r#Bank1Info2RegwenRead;
type Write = r#Bank1Info2RegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info2RegwenRead { pub reg: register::RegRead<r#Bank1Info2Regwen> }
impl r#Bank1Info2RegwenRead {
#[inline]
pub fn r#region(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info2RegwenWrite { pub reg: register::RegWrite<r#Bank1Info2Regwen> }
impl r#Bank1Info2RegwenWrite {
#[inline]
pub fn r#region(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Bank1Info2PageCfg {}
impl register::RegSpec for r#Bank1Info2PageCfg {
const DEFAULT: u32 = 0x9999999;
type Read = r#Bank1Info2PageCfgRead;
type Write = r#Bank1Info2PageCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info2PageCfgRead { pub reg: register::RegRead<r#Bank1Info2PageCfg> }
impl r#Bank1Info2PageCfgRead {
#[inline]
pub fn r#en(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#rd_en(self) -> u32 {
self.reg.field(0xf0)
}
#[inline]
pub fn r#prog_en(self) -> u32 {
self.reg.field(0xf00)
}
#[inline]
pub fn r#erase_en(self) -> u32 {
self.reg.field(0xf000)
}
#[inline]
pub fn r#scramble_en(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#ecc_en(self) -> u32 {
self.reg.field(0xf00000)
}
#[inline]
pub fn r#he_en(self) -> u32 {
self.reg.field(0xf000000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Bank1Info2PageCfgWrite { pub reg: register::RegWrite<r#Bank1Info2PageCfg> }
impl r#Bank1Info2PageCfgWrite {
#[inline]
pub fn r#en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#rd_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
#[inline]
pub fn r#prog_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00, value); self
}
#[inline]
pub fn r#erase_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000, value); self
}
#[inline]
pub fn r#scramble_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#ecc_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
#[inline]
pub fn r#he_en(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
}
pub enum r#HwInfoCfgOverride {}
impl register::RegSpec for r#HwInfoCfgOverride {
const DEFAULT: u32 = 0x99;
type Read = r#HwInfoCfgOverrideRead;
type Write = r#HwInfoCfgOverrideWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#HwInfoCfgOverrideRead { pub reg: register::RegRead<r#HwInfoCfgOverride> }
impl r#HwInfoCfgOverrideRead {
#[inline]
pub fn r#scramble_dis(self) -> u32 {
self.reg.field(0xf)
}
#[inline]
pub fn r#ecc_dis(self) -> u32 {
self.reg.field(0xf0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#HwInfoCfgOverrideWrite { pub reg: register::RegWrite<r#HwInfoCfgOverride> }
impl r#HwInfoCfgOverrideWrite {
#[inline]
pub fn r#scramble_dis(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
#[inline]
pub fn r#ecc_dis(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
}
pub enum r#BankCfgRegwen {}
impl register::RegSpec for r#BankCfgRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#BankCfgRegwenRead;
type Write = r#BankCfgRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#BankCfgRegwenRead { pub reg: register::RegRead<r#BankCfgRegwen> }
impl r#BankCfgRegwenRead {
#[inline]
pub fn r#bank(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#BankCfgRegwenWrite { pub reg: register::RegWrite<r#BankCfgRegwen> }
impl r#BankCfgRegwenWrite {
#[inline]
pub fn r#bank(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MpBankCfgShadowed {}
impl register::RegSpec for r#MpBankCfgShadowed {
const DEFAULT: u32 = 0x0;
type Read = r#MpBankCfgShadowedRead;
type Write = r#MpBankCfgShadowedWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MpBankCfgShadowedRead { pub reg: register::RegRead<r#MpBankCfgShadowed> }
impl r#MpBankCfgShadowedRead {
#[inline]
pub fn r#erase_en(self, index: u8) -> bool {
assert!(index < 2);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MpBankCfgShadowedWrite { pub reg: register::RegWrite<r#MpBankCfgShadowed> }
impl r#MpBankCfgShadowedWrite {
#[inline]
pub fn r#erase_en(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 2);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#OpStatus {}
impl register::RegSpec for r#OpStatus {
const DEFAULT: u32 = 0x0;
type Read = r#OpStatusRead;
type Write = r#OpStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OpStatusRead { pub reg: register::RegRead<r#OpStatus> }
impl r#OpStatusRead {
#[inline]
pub fn r#done(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#err(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OpStatusWrite { pub reg: register::RegWrite<r#OpStatus> }
impl r#OpStatusWrite {
#[inline]
pub fn r#done(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#err(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#Status {}
impl register::RegSpec for r#Status {
const DEFAULT: u32 = 0xa;
type Read = r#StatusRead;
type Write = r#StatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusRead { pub reg: register::RegRead<r#Status> }
impl r#StatusRead {
#[inline]
pub fn r#rd_full(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#rd_empty(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#prog_full(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#prog_empty(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#init_wip(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#initialized(self) -> bool {
self.reg.bit(5)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusWrite { pub reg: register::RegWrite<r#Status> }
impl r#StatusWrite {
#[inline]
pub fn r#rd_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#rd_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#prog_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#prog_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#init_wip(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#initialized(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
}
pub enum r#DebugState {}
impl register::RegSpec for r#DebugState {
const DEFAULT: u32 = 0x0;
type Read = r#DebugStateRead;
type Write = r#DebugStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DebugStateRead { pub reg: register::RegRead<r#DebugState> }
impl r#DebugStateRead {
#[inline]
pub fn r#lcmgr_state(self) -> u32 {
self.reg.field(0x7ff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DebugStateWrite { pub reg: register::RegWrite<r#DebugState> }
impl r#DebugStateWrite {
#[inline]
pub fn r#lcmgr_state(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7ff, value); self
}
}
pub enum r#ErrCode {}
impl register::RegSpec for r#ErrCode {
const DEFAULT: u32 = 0x0;
type Read = r#ErrCodeRead;
type Write = r#ErrCodeWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrCodeRead { pub reg: register::RegRead<r#ErrCode> }
impl r#ErrCodeRead {
#[inline]
pub fn r#op_err(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#mp_err(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#rd_err(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#prog_err(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#prog_win_err(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#prog_type_err(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#update_err(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#macro_err(self) -> bool {
self.reg.bit(7)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrCodeWrite { pub reg: register::RegWrite<r#ErrCode> }
impl r#ErrCodeWrite {
#[inline]
pub fn r#op_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#mp_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#rd_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#prog_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#prog_win_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#prog_type_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#update_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#macro_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
}
pub enum r#StdFaultStatus {}
impl register::RegSpec for r#StdFaultStatus {
const DEFAULT: u32 = 0x0;
type Read = r#StdFaultStatusRead;
type Write = r#StdFaultStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StdFaultStatusRead { pub reg: register::RegRead<r#StdFaultStatus> }
impl r#StdFaultStatusRead {
#[inline]
pub fn r#reg_intg_err(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#prog_intg_err(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#lcmgr_err(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#lcmgr_intg_err(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#arb_fsm_err(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#storage_err(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#phy_fsm_err(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#ctrl_cnt_err(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#fifo_err(self) -> bool {
self.reg.bit(8)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StdFaultStatusWrite { pub reg: register::RegWrite<r#StdFaultStatus> }
impl r#StdFaultStatusWrite {
#[inline]
pub fn r#reg_intg_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#prog_intg_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#lcmgr_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#lcmgr_intg_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#arb_fsm_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#storage_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#phy_fsm_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#ctrl_cnt_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#fifo_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
}
pub enum r#FaultStatus {}
impl register::RegSpec for r#FaultStatus {
const DEFAULT: u32 = 0x0;
type Read = r#FaultStatusRead;
type Write = r#FaultStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FaultStatusRead { pub reg: register::RegRead<r#FaultStatus> }
impl r#FaultStatusRead {
#[inline]
pub fn r#op_err(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#mp_err(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#rd_err(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#prog_err(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#prog_win_err(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#prog_type_err(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#seed_err(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#phy_relbl_err(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#phy_storage_err(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#spurious_ack(self) -> bool {
self.reg.bit(9)
}
#[inline]
pub fn r#arb_err(self) -> bool {
self.reg.bit(10)
}
#[inline]
pub fn r#host_gnt_err(self) -> bool {
self.reg.bit(11)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FaultStatusWrite { pub reg: register::RegWrite<r#FaultStatus> }
impl r#FaultStatusWrite {
#[inline]
pub fn r#op_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#mp_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#rd_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#prog_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#prog_win_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#prog_type_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#seed_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#phy_relbl_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#phy_storage_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#spurious_ack(&mut self, value: bool) -> &mut Self {
self.reg.bit(9, value); self
}
#[inline]
pub fn r#arb_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(10, value); self
}
#[inline]
pub fn r#host_gnt_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(11, value); self
}
}
pub enum r#ErrAddr {}
impl register::RegSpec for r#ErrAddr {
const DEFAULT: u32 = 0x0;
type Read = r#ErrAddrRead;
type Write = r#ErrAddrWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrAddrRead { pub reg: register::RegRead<r#ErrAddr> }
impl r#ErrAddrRead {
#[inline]
pub fn r#err_addr(self) -> u32 {
self.reg.field(0xfffff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrAddrWrite { pub reg: register::RegWrite<r#ErrAddr> }
impl r#ErrAddrWrite {
#[inline]
pub fn r#err_addr(&mut self, value: u32) -> &mut Self {
self.reg.field(0xfffff, value); self
}
}
pub enum r#EccSingleErrCnt {}
impl register::RegSpec for r#EccSingleErrCnt {
const DEFAULT: u32 = 0x0;
type Read = r#EccSingleErrCntRead;
type Write = r#EccSingleErrCntWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EccSingleErrCntRead { pub reg: register::RegRead<r#EccSingleErrCnt> }
impl r#EccSingleErrCntRead {
#[inline]
pub fn r#ecc_single_err_cnt_0(self) -> u32 {
self.reg.field(0xff)
}
#[inline]
pub fn r#ecc_single_err_cnt_1(self) -> u32 {
self.reg.field(0xff00)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EccSingleErrCntWrite { pub reg: register::RegWrite<r#EccSingleErrCnt> }
impl r#EccSingleErrCntWrite {
#[inline]
pub fn r#ecc_single_err_cnt_0(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
#[inline]
pub fn r#ecc_single_err_cnt_1(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff00, value); self
}
}
pub enum r#EccSingleErrAddr {}
impl register::RegSpec for r#EccSingleErrAddr {
const DEFAULT: u32 = 0x0;
type Read = r#EccSingleErrAddrRead;
type Write = r#EccSingleErrAddrWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EccSingleErrAddrRead { pub reg: register::RegRead<r#EccSingleErrAddr> }
impl r#EccSingleErrAddrRead {
#[inline]
pub fn r#ecc_single_err_addr(self) -> u32 {
self.reg.field(0xfffff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EccSingleErrAddrWrite { pub reg: register::RegWrite<r#EccSingleErrAddr> }
impl r#EccSingleErrAddrWrite {
#[inline]
pub fn r#ecc_single_err_addr(&mut self, value: u32) -> &mut Self {
self.reg.field(0xfffff, value); self
}
}
pub enum r#PhyAlertCfg {}
impl register::RegSpec for r#PhyAlertCfg {
const DEFAULT: u32 = 0x0;
type Read = r#PhyAlertCfgRead;
type Write = r#PhyAlertCfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyAlertCfgRead { pub reg: register::RegRead<r#PhyAlertCfg> }
impl r#PhyAlertCfgRead {
#[inline]
pub fn r#alert_ack(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#alert_trig(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyAlertCfgWrite { pub reg: register::RegWrite<r#PhyAlertCfg> }
impl r#PhyAlertCfgWrite {
#[inline]
pub fn r#alert_ack(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#alert_trig(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#PhyStatus {}
impl register::RegSpec for r#PhyStatus {
const DEFAULT: u32 = 0x6;
type Read = r#PhyStatusRead;
type Write = r#PhyStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyStatusRead { pub reg: register::RegRead<r#PhyStatus> }
impl r#PhyStatusRead {
#[inline]
pub fn r#init_wip(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#prog_normal_avail(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#prog_repair_avail(self) -> bool {
self.reg.bit(2)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyStatusWrite { pub reg: register::RegWrite<r#PhyStatus> }
impl r#PhyStatusWrite {
#[inline]
pub fn r#init_wip(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#prog_normal_avail(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#prog_repair_avail(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
}
pub enum r#Scratch {}
impl register::RegSpec for r#Scratch {
const DEFAULT: u32 = 0x0;
type Read = r#ScratchRead;
type Write = r#ScratchWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ScratchRead { pub reg: register::RegRead<r#Scratch> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ScratchWrite { pub reg: register::RegWrite<r#Scratch> }
pub enum r#FifoLvl {}
impl register::RegSpec for r#FifoLvl {
const DEFAULT: u32 = 0xf0f;
type Read = r#FifoLvlRead;
type Write = r#FifoLvlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoLvlRead { pub reg: register::RegRead<r#FifoLvl> }
impl r#FifoLvlRead {
#[inline]
pub fn r#prog(self) -> u32 {
self.reg.field(0x1f)
}
#[inline]
pub fn r#rd(self) -> u32 {
self.reg.field(0x1f00)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoLvlWrite { pub reg: register::RegWrite<r#FifoLvl> }
impl r#FifoLvlWrite {
#[inline]
pub fn r#prog(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f, value); self
}
#[inline]
pub fn r#rd(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f00, value); self
}
}
pub enum r#FifoRst {}
impl register::RegSpec for r#FifoRst {
const DEFAULT: u32 = 0x0;
type Read = r#FifoRstRead;
type Write = r#FifoRstWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoRstRead { pub reg: register::RegRead<r#FifoRst> }
impl r#FifoRstRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoRstWrite { pub reg: register::RegWrite<r#FifoRst> }
impl r#FifoRstWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#CurrFifoLvl {}
impl register::RegSpec for r#CurrFifoLvl {
const DEFAULT: u32 = 0x0;
type Read = r#CurrFifoLvlRead;
type Write = r#CurrFifoLvlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CurrFifoLvlRead { pub reg: register::RegRead<r#CurrFifoLvl> }
impl r#CurrFifoLvlRead {
#[inline]
pub fn r#prog(self) -> u32 {
self.reg.field(0x1f)
}
#[inline]
pub fn r#rd(self) -> u32 {
self.reg.field(0x1f00)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CurrFifoLvlWrite { pub reg: register::RegWrite<r#CurrFifoLvl> }
impl r#CurrFifoLvlWrite {
#[inline]
pub fn r#prog(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f, value); self
}
#[inline]
pub fn r#rd(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f00, value); self
}
}
pub enum r#ProgFifo {}
impl register::RegSpec for r#ProgFifo {
const DEFAULT: u32 = 0x0;
type Read = r#ProgFifoRead;
type Write = r#ProgFifoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ProgFifoRead { pub reg: register::RegRead<r#ProgFifo> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ProgFifoWrite { pub reg: register::RegWrite<r#ProgFifo> }
pub enum r#RdFifo {}
impl register::RegSpec for r#RdFifo {
const DEFAULT: u32 = 0x0;
type Read = r#RdFifoRead;
type Write = r#RdFifoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RdFifoRead { pub reg: register::RegRead<r#RdFifo> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RdFifoWrite { pub reg: register::RegWrite<r#RdFifo> }
}
#[rustfmt::skip]
pub mod r#hmac {
impl super::r#Hmac {
#[inline]
pub fn r#intr_state(self) -> register::RegAddr<r#IntrState> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#intr_enable(self) -> register::RegAddr<r#IntrEnable> {
unsafe { register::RegAddr::new(self.addr + 0x4) }
}
#[inline]
pub fn r#intr_test(self) -> register::RegAddr<r#IntrTest> {
unsafe { register::RegAddr::new(self.addr + 0x8) }
}
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0xc) }
}
#[inline]
pub fn r#cfg(self) -> register::RegAddr<r#Cfg> {
unsafe { register::RegAddr::new(self.addr + 0x10) }
}
#[inline]
pub fn r#cmd(self) -> register::RegAddr<r#Cmd> {
unsafe { register::RegAddr::new(self.addr + 0x14) }
}
#[inline]
pub fn r#status(self) -> register::RegAddr<r#Status> {
unsafe { register::RegAddr::new(self.addr + 0x18) }
}
#[inline]
pub fn r#err_code(self) -> register::RegAddr<r#ErrCode> {
unsafe { register::RegAddr::new(self.addr + 0x1c) }
}
#[inline]
pub fn r#wipe_secret(self) -> register::RegAddr<r#WipeSecret> {
unsafe { register::RegAddr::new(self.addr + 0x20) }
}
#[inline]
pub fn r#key(self, index: u32) -> register::RegAddr<r#Key> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x24 + index * 4) }
}
#[inline]
pub fn r#digest(self, index: u32) -> register::RegAddr<r#Digest> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x44 + index * 4) }
}
#[inline]
pub fn r#msg_length_lower(self) -> register::RegAddr<r#MsgLengthLower> {
unsafe { register::RegAddr::new(self.addr + 0x64) }
}
#[inline]
pub fn r#msg_length_upper(self) -> register::RegAddr<r#MsgLengthUpper> {
unsafe { register::RegAddr::new(self.addr + 0x68) }
}
#[inline]
pub fn r#msg_fifo(self, index: u32) -> register::RegAddr<r#MsgFifo> {
assert!(index < 512);
unsafe { register::RegAddr::new(self.addr + 0x800 + index * 4) }
}
}
pub enum r#IntrState {}
impl register::RegSpec for r#IntrState {
const DEFAULT: u32 = 0x0;
type Read = r#IntrStateRead;
type Write = r#IntrStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateRead { pub reg: register::RegRead<r#IntrState> }
impl r#IntrStateRead {
#[inline]
pub fn r#hmac_done(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#fifo_empty(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#hmac_err(self) -> bool {
self.reg.bit(2)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateWrite { pub reg: register::RegWrite<r#IntrState> }
impl r#IntrStateWrite {
#[inline]
pub fn r#hmac_done(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#fifo_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#hmac_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
}
pub enum r#IntrEnable {}
impl register::RegSpec for r#IntrEnable {
const DEFAULT: u32 = 0x0;
type Read = r#IntrEnableRead;
type Write = r#IntrEnableWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableRead { pub reg: register::RegRead<r#IntrEnable> }
impl r#IntrEnableRead {
#[inline]
pub fn r#hmac_done(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#fifo_empty(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#hmac_err(self) -> bool {
self.reg.bit(2)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableWrite { pub reg: register::RegWrite<r#IntrEnable> }
impl r#IntrEnableWrite {
#[inline]
pub fn r#hmac_done(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#fifo_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#hmac_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
}
pub enum r#IntrTest {}
impl register::RegSpec for r#IntrTest {
const DEFAULT: u32 = 0x0;
type Read = r#IntrTestRead;
type Write = r#IntrTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestRead { pub reg: register::RegRead<r#IntrTest> }
impl r#IntrTestRead {
#[inline]
pub fn r#hmac_done(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#fifo_empty(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#hmac_err(self) -> bool {
self.reg.bit(2)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestWrite { pub reg: register::RegWrite<r#IntrTest> }
impl r#IntrTestWrite {
#[inline]
pub fn r#hmac_done(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#fifo_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#hmac_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_fault(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Cfg {}
impl register::RegSpec for r#Cfg {
const DEFAULT: u32 = 0x0;
type Read = r#CfgRead;
type Write = r#CfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CfgRead { pub reg: register::RegRead<r#Cfg> }
impl r#CfgRead {
#[inline]
pub fn r#hmac_en(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#sha_en(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#endian_swap(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#digest_swap(self) -> bool {
self.reg.bit(3)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CfgWrite { pub reg: register::RegWrite<r#Cfg> }
impl r#CfgWrite {
#[inline]
pub fn r#hmac_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#sha_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#endian_swap(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#digest_swap(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
}
pub enum r#Cmd {}
impl register::RegSpec for r#Cmd {
const DEFAULT: u32 = 0x0;
type Read = r#CmdRead;
type Write = r#CmdWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CmdRead { pub reg: register::RegRead<r#Cmd> }
impl r#CmdRead {
#[inline]
pub fn r#hash_start(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#hash_process(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CmdWrite { pub reg: register::RegWrite<r#Cmd> }
impl r#CmdWrite {
#[inline]
pub fn r#hash_start(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#hash_process(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#Status {}
impl register::RegSpec for r#Status {
const DEFAULT: u32 = 0x1;
type Read = r#StatusRead;
type Write = r#StatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusRead { pub reg: register::RegRead<r#Status> }
impl r#StatusRead {
#[inline]
pub fn r#fifo_empty(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#fifo_full(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#fifo_depth(self) -> u32 {
self.reg.field(0x1f0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusWrite { pub reg: register::RegWrite<r#Status> }
impl r#StatusWrite {
#[inline]
pub fn r#fifo_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#fifo_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#fifo_depth(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f0, value); self
}
}
pub enum r#ErrCode {}
impl register::RegSpec for r#ErrCode {
const DEFAULT: u32 = 0x0;
type Read = r#ErrCodeRead;
type Write = r#ErrCodeWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrCodeRead { pub reg: register::RegRead<r#ErrCode> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrCodeWrite { pub reg: register::RegWrite<r#ErrCode> }
pub enum r#WipeSecret {}
impl register::RegSpec for r#WipeSecret {
const DEFAULT: u32 = 0x0;
type Read = r#WipeSecretRead;
type Write = r#WipeSecretWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WipeSecretRead { pub reg: register::RegRead<r#WipeSecret> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WipeSecretWrite { pub reg: register::RegWrite<r#WipeSecret> }
pub enum r#Key {}
impl register::RegSpec for r#Key {
const DEFAULT: u32 = 0x0;
type Read = r#KeyRead;
type Write = r#KeyWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#KeyRead { pub reg: register::RegRead<r#Key> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#KeyWrite { pub reg: register::RegWrite<r#Key> }
pub enum r#Digest {}
impl register::RegSpec for r#Digest {
const DEFAULT: u32 = 0x0;
type Read = r#DigestRead;
type Write = r#DigestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DigestRead { pub reg: register::RegRead<r#Digest> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DigestWrite { pub reg: register::RegWrite<r#Digest> }
pub enum r#MsgLengthLower {}
impl register::RegSpec for r#MsgLengthLower {
const DEFAULT: u32 = 0x0;
type Read = r#MsgLengthLowerRead;
type Write = r#MsgLengthLowerWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsgLengthLowerRead { pub reg: register::RegRead<r#MsgLengthLower> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsgLengthLowerWrite { pub reg: register::RegWrite<r#MsgLengthLower> }
pub enum r#MsgLengthUpper {}
impl register::RegSpec for r#MsgLengthUpper {
const DEFAULT: u32 = 0x0;
type Read = r#MsgLengthUpperRead;
type Write = r#MsgLengthUpperWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsgLengthUpperRead { pub reg: register::RegRead<r#MsgLengthUpper> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsgLengthUpperWrite { pub reg: register::RegWrite<r#MsgLengthUpper> }
pub enum r#MsgFifo {}
impl register::RegSpec for r#MsgFifo {
const DEFAULT: u32 = 0x0;
type Read = r#MsgFifoRead;
type Write = r#MsgFifoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsgFifoRead { pub reg: register::RegRead<r#MsgFifo> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsgFifoWrite { pub reg: register::RegWrite<r#MsgFifo> }
}
#[rustfmt::skip]
pub mod r#lc_ctrl {
impl super::r#LcCtrl {
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#status(self) -> register::RegAddr<r#Status> {
unsafe { register::RegAddr::new(self.addr + 0x4) }
}
#[inline]
pub fn r#claim_transition_if_regwen(self) -> register::RegAddr<r#ClaimTransitionIfRegwen> {
unsafe { register::RegAddr::new(self.addr + 0x8) }
}
#[inline]
pub fn r#claim_transition_if(self) -> register::RegAddr<r#ClaimTransitionIf> {
unsafe { register::RegAddr::new(self.addr + 0xc) }
}
#[inline]
pub fn r#transition_regwen(self) -> register::RegAddr<r#TransitionRegwen> {
unsafe { register::RegAddr::new(self.addr + 0x10) }
}
#[inline]
pub fn r#transition_cmd(self) -> register::RegAddr<r#TransitionCmd> {
unsafe { register::RegAddr::new(self.addr + 0x14) }
}
#[inline]
pub fn r#transition_ctrl(self) -> register::RegAddr<r#TransitionCtrl> {
unsafe { register::RegAddr::new(self.addr + 0x18) }
}
#[inline]
pub fn r#transition_token(self, index: u32) -> register::RegAddr<r#TransitionToken> {
assert!(index < 4);
unsafe { register::RegAddr::new(self.addr + 0x1c + index * 4) }
}
#[inline]
pub fn r#transition_target(self) -> register::RegAddr<r#TransitionTarget> {
unsafe { register::RegAddr::new(self.addr + 0x2c) }
}
#[inline]
pub fn r#otp_vendor_test_ctrl(self) -> register::RegAddr<r#OtpVendorTestCtrl> {
unsafe { register::RegAddr::new(self.addr + 0x30) }
}
#[inline]
pub fn r#otp_vendor_test_status(self) -> register::RegAddr<r#OtpVendorTestStatus> {
unsafe { register::RegAddr::new(self.addr + 0x34) }
}
#[inline]
pub fn r#lc_state(self) -> register::RegAddr<r#LcState> {
unsafe { register::RegAddr::new(self.addr + 0x38) }
}
#[inline]
pub fn r#lc_transition_cnt(self) -> register::RegAddr<r#LcTransitionCnt> {
unsafe { register::RegAddr::new(self.addr + 0x3c) }
}
#[inline]
pub fn r#lc_id_state(self) -> register::RegAddr<r#LcIdState> {
unsafe { register::RegAddr::new(self.addr + 0x40) }
}
#[inline]
pub fn r#hw_revision0(self) -> register::RegAddr<r#HwRevision0> {
unsafe { register::RegAddr::new(self.addr + 0x44) }
}
#[inline]
pub fn r#hw_revision1(self) -> register::RegAddr<r#HwRevision1> {
unsafe { register::RegAddr::new(self.addr + 0x48) }
}
#[inline]
pub fn r#device_id(self, index: u32) -> register::RegAddr<r#DeviceId> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x4c + index * 4) }
}
#[inline]
pub fn r#manuf_state(self, index: u32) -> register::RegAddr<r#ManufState> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x6c + index * 4) }
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_prog_error(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#fatal_state_error(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#fatal_bus_integ_error(self) -> bool {
self.reg.bit(2)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_prog_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#fatal_state_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#fatal_bus_integ_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
}
pub enum r#Status {}
impl register::RegSpec for r#Status {
const DEFAULT: u32 = 0x0;
type Read = r#StatusRead;
type Write = r#StatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusRead { pub reg: register::RegRead<r#Status> }
impl r#StatusRead {
#[inline]
pub fn r#initialized(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#ready(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#ext_clock_switched(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#transition_successful(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#transition_count_error(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#transition_error(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#token_error(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#flash_rma_error(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#otp_error(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#state_error(self) -> bool {
self.reg.bit(9)
}
#[inline]
pub fn r#bus_integ_error(self) -> bool {
self.reg.bit(10)
}
#[inline]
pub fn r#otp_partition_error(self) -> bool {
self.reg.bit(11)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusWrite { pub reg: register::RegWrite<r#Status> }
impl r#StatusWrite {
#[inline]
pub fn r#initialized(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#ready(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#ext_clock_switched(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#transition_successful(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#transition_count_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#transition_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#token_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#flash_rma_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#otp_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#state_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(9, value); self
}
#[inline]
pub fn r#bus_integ_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(10, value); self
}
#[inline]
pub fn r#otp_partition_error(&mut self, value: bool) -> &mut Self {
self.reg.bit(11, value); self
}
}
pub enum r#ClaimTransitionIfRegwen {}
impl register::RegSpec for r#ClaimTransitionIfRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#ClaimTransitionIfRegwenRead;
type Write = r#ClaimTransitionIfRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ClaimTransitionIfRegwenRead { pub reg: register::RegRead<r#ClaimTransitionIfRegwen> }
impl r#ClaimTransitionIfRegwenRead {
#[inline]
pub fn r#claim_transition_if_regwen(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ClaimTransitionIfRegwenWrite { pub reg: register::RegWrite<r#ClaimTransitionIfRegwen> }
impl r#ClaimTransitionIfRegwenWrite {
#[inline]
pub fn r#claim_transition_if_regwen(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#ClaimTransitionIf {}
impl register::RegSpec for r#ClaimTransitionIf {
const DEFAULT: u32 = 0x69;
type Read = r#ClaimTransitionIfRead;
type Write = r#ClaimTransitionIfWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ClaimTransitionIfRead { pub reg: register::RegRead<r#ClaimTransitionIf> }
impl r#ClaimTransitionIfRead {
#[inline]
pub fn r#mutex(self) -> u32 {
self.reg.field(0xff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ClaimTransitionIfWrite { pub reg: register::RegWrite<r#ClaimTransitionIf> }
impl r#ClaimTransitionIfWrite {
#[inline]
pub fn r#mutex(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
}
pub enum r#TransitionRegwen {}
impl register::RegSpec for r#TransitionRegwen {
const DEFAULT: u32 = 0x0;
type Read = r#TransitionRegwenRead;
type Write = r#TransitionRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionRegwenRead { pub reg: register::RegRead<r#TransitionRegwen> }
impl r#TransitionRegwenRead {
#[inline]
pub fn r#transition_regwen(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionRegwenWrite { pub reg: register::RegWrite<r#TransitionRegwen> }
impl r#TransitionRegwenWrite {
#[inline]
pub fn r#transition_regwen(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#TransitionCmd {}
impl register::RegSpec for r#TransitionCmd {
const DEFAULT: u32 = 0x0;
type Read = r#TransitionCmdRead;
type Write = r#TransitionCmdWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionCmdRead { pub reg: register::RegRead<r#TransitionCmd> }
impl r#TransitionCmdRead {
#[inline]
pub fn r#start(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionCmdWrite { pub reg: register::RegWrite<r#TransitionCmd> }
impl r#TransitionCmdWrite {
#[inline]
pub fn r#start(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#TransitionCtrl {}
impl register::RegSpec for r#TransitionCtrl {
const DEFAULT: u32 = 0x0;
type Read = r#TransitionCtrlRead;
type Write = r#TransitionCtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionCtrlRead { pub reg: register::RegRead<r#TransitionCtrl> }
impl r#TransitionCtrlRead {
#[inline]
pub fn r#ext_clock_en(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#volatile_raw_unlock(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionCtrlWrite { pub reg: register::RegWrite<r#TransitionCtrl> }
impl r#TransitionCtrlWrite {
#[inline]
pub fn r#ext_clock_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#volatile_raw_unlock(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#TransitionToken {}
impl register::RegSpec for r#TransitionToken {
const DEFAULT: u32 = 0x0;
type Read = r#TransitionTokenRead;
type Write = r#TransitionTokenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionTokenRead { pub reg: register::RegRead<r#TransitionToken> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionTokenWrite { pub reg: register::RegWrite<r#TransitionToken> }
pub enum r#TransitionTarget {}
impl register::RegSpec for r#TransitionTarget {
const DEFAULT: u32 = 0x0;
type Read = r#TransitionTargetRead;
type Write = r#TransitionTargetWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionTargetRead { pub reg: register::RegRead<r#TransitionTarget> }
impl r#TransitionTargetRead {
#[inline]
pub fn r#state(self) -> u32 {
self.reg.field(0x3fffffff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TransitionTargetWrite { pub reg: register::RegWrite<r#TransitionTarget> }
impl r#TransitionTargetWrite {
#[inline]
pub fn r#state(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3fffffff, value); self
}
}
pub mod r#transition_target {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#State {
r#Raw = 0x0,
r#TestUnlocked0 = 0x2108421,
r#TestLocked0 = 0x4210842,
r#TestUnlocked1 = 0x6318c63,
r#TestLocked1 = 0x8421084,
r#TestUnlocked2 = 0xa5294a5,
r#TestLocked2 = 0xc6318c6,
r#TestUnlocked3 = 0xe739ce7,
r#TestLocked3 = 0x10842108,
r#TestUnlocked4 = 0x1294a529,
r#TestLocked4 = 0x14a5294a,
r#TestUnlocked5 = 0x16b5ad6b,
r#TestLocked5 = 0x18c6318c,
r#TestUnlocked6 = 0x1ad6b5ad,
r#TestLocked6 = 0x1ce739ce,
r#TestUnlocked7 = 0x1ef7bdef,
r#Dev = 0x21084210,
r#Prod = 0x2318c631,
r#ProdEnd = 0x25294a52,
r#Rma = 0x2739ce73,
r#Scrap = 0x294a5294,
}
}
pub enum r#OtpVendorTestCtrl {}
impl register::RegSpec for r#OtpVendorTestCtrl {
const DEFAULT: u32 = 0x0;
type Read = r#OtpVendorTestCtrlRead;
type Write = r#OtpVendorTestCtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OtpVendorTestCtrlRead { pub reg: register::RegRead<r#OtpVendorTestCtrl> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OtpVendorTestCtrlWrite { pub reg: register::RegWrite<r#OtpVendorTestCtrl> }
pub enum r#OtpVendorTestStatus {}
impl register::RegSpec for r#OtpVendorTestStatus {
const DEFAULT: u32 = 0x0;
type Read = r#OtpVendorTestStatusRead;
type Write = r#OtpVendorTestStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OtpVendorTestStatusRead { pub reg: register::RegRead<r#OtpVendorTestStatus> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OtpVendorTestStatusWrite { pub reg: register::RegWrite<r#OtpVendorTestStatus> }
pub enum r#LcState {}
impl register::RegSpec for r#LcState {
const DEFAULT: u32 = 0x0;
type Read = r#LcStateRead;
type Write = r#LcStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#LcStateRead { pub reg: register::RegRead<r#LcState> }
impl r#LcStateRead {
#[inline]
pub fn r#state(self) -> u32 {
self.reg.field(0x3fffffff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#LcStateWrite { pub reg: register::RegWrite<r#LcState> }
impl r#LcStateWrite {
#[inline]
pub fn r#state(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3fffffff, value); self
}
}
pub mod r#lc_state {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#State {
r#Raw = 0x0,
r#TestUnlocked0 = 0x2108421,
r#TestLocked0 = 0x4210842,
r#TestUnlocked1 = 0x6318c63,
r#TestLocked1 = 0x8421084,
r#TestUnlocked2 = 0xa5294a5,
r#TestLocked2 = 0xc6318c6,
r#TestUnlocked3 = 0xe739ce7,
r#TestLocked3 = 0x10842108,
r#TestUnlocked4 = 0x1294a529,
r#TestLocked4 = 0x14a5294a,
r#TestUnlocked5 = 0x16b5ad6b,
r#TestLocked5 = 0x18c6318c,
r#TestUnlocked6 = 0x1ad6b5ad,
r#TestLocked6 = 0x1ce739ce,
r#TestUnlocked7 = 0x1ef7bdef,
r#Dev = 0x21084210,
r#Prod = 0x2318c631,
r#ProdEnd = 0x25294a52,
r#Rma = 0x2739ce73,
r#Scrap = 0x294a5294,
r#PostTransition = 0x2b5ad6b5,
r#Escalate = 0x2d6b5ad6,
r#Invalid = 0x2f7bdef7,
}
}
pub enum r#LcTransitionCnt {}
impl register::RegSpec for r#LcTransitionCnt {
const DEFAULT: u32 = 0x0;
type Read = r#LcTransitionCntRead;
type Write = r#LcTransitionCntWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#LcTransitionCntRead { pub reg: register::RegRead<r#LcTransitionCnt> }
impl r#LcTransitionCntRead {
#[inline]
pub fn r#cnt(self) -> u32 {
self.reg.field(0x1f)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#LcTransitionCntWrite { pub reg: register::RegWrite<r#LcTransitionCnt> }
impl r#LcTransitionCntWrite {
#[inline]
pub fn r#cnt(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f, value); self
}
}
pub enum r#LcIdState {}
impl register::RegSpec for r#LcIdState {
const DEFAULT: u32 = 0x0;
type Read = r#LcIdStateRead;
type Write = r#LcIdStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#LcIdStateRead { pub reg: register::RegRead<r#LcIdState> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#LcIdStateWrite { pub reg: register::RegWrite<r#LcIdState> }
pub enum r#HwRevision0 {}
impl register::RegSpec for r#HwRevision0 {
const DEFAULT: u32 = 0x0;
type Read = r#HwRevision0Read;
type Write = r#HwRevision0Write;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#HwRevision0Read { pub reg: register::RegRead<r#HwRevision0> }
impl r#HwRevision0Read {
#[inline]
pub fn r#product_id(self) -> u32 {
self.reg.field(0xffff)
}
#[inline]
pub fn r#silicon_creator_id(self) -> u32 {
self.reg.field(0xffff0000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#HwRevision0Write { pub reg: register::RegWrite<r#HwRevision0> }
impl r#HwRevision0Write {
#[inline]
pub fn r#product_id(&mut self, value: u32) -> &mut Self {
self.reg.field(0xffff, value); self
}
#[inline]
pub fn r#silicon_creator_id(&mut self, value: u32) -> &mut Self {
self.reg.field(0xffff0000, value); self
}
}
pub enum r#HwRevision1 {}
impl register::RegSpec for r#HwRevision1 {
const DEFAULT: u32 = 0x0;
type Read = r#HwRevision1Read;
type Write = r#HwRevision1Write;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#HwRevision1Read { pub reg: register::RegRead<r#HwRevision1> }
impl r#HwRevision1Read {
#[inline]
pub fn r#revision_id(self) -> u32 {
self.reg.field(0xff)
}
#[inline]
pub fn r#reserved(self) -> u32 {
self.reg.field(0xffffff00)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#HwRevision1Write { pub reg: register::RegWrite<r#HwRevision1> }
impl r#HwRevision1Write {
#[inline]
pub fn r#revision_id(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
#[inline]
pub fn r#reserved(&mut self, value: u32) -> &mut Self {
self.reg.field(0xffffff00, value); self
}
}
pub enum r#DeviceId {}
impl register::RegSpec for r#DeviceId {
const DEFAULT: u32 = 0x0;
type Read = r#DeviceIdRead;
type Write = r#DeviceIdWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DeviceIdRead { pub reg: register::RegRead<r#DeviceId> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DeviceIdWrite { pub reg: register::RegWrite<r#DeviceId> }
pub enum r#ManufState {}
impl register::RegSpec for r#ManufState {
const DEFAULT: u32 = 0x0;
type Read = r#ManufStateRead;
type Write = r#ManufStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ManufStateRead { pub reg: register::RegRead<r#ManufState> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ManufStateWrite { pub reg: register::RegWrite<r#ManufState> }
}
#[rustfmt::skip]
pub mod r#pinmux {
impl super::r#Pinmux {
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#mio_periph_insel_regwen(self, index: u32) -> register::RegAddr<r#MioPeriphInselRegwen> {
assert!(index < 57);
unsafe { register::RegAddr::new(self.addr + 0x4 + index * 4) }
}
#[inline]
pub fn r#mio_periph_insel(self, index: u32) -> register::RegAddr<r#MioPeriphInsel> {
assert!(index < 57);
unsafe { register::RegAddr::new(self.addr + 0xe8 + index * 4) }
}
#[inline]
pub fn r#mio_outsel_regwen(self, index: u32) -> register::RegAddr<r#MioOutselRegwen> {
assert!(index < 47);
unsafe { register::RegAddr::new(self.addr + 0x1cc + index * 4) }
}
#[inline]
pub fn r#mio_outsel(self, index: u32) -> register::RegAddr<r#MioOutsel> {
assert!(index < 47);
unsafe { register::RegAddr::new(self.addr + 0x288 + index * 4) }
}
#[inline]
pub fn r#mio_pad_attr_regwen(self, index: u32) -> register::RegAddr<r#MioPadAttrRegwen> {
assert!(index < 47);
unsafe { register::RegAddr::new(self.addr + 0x344 + index * 4) }
}
#[inline]
pub fn r#mio_pad_attr(self, index: u32) -> register::RegAddr<r#MioPadAttr> {
assert!(index < 47);
unsafe { register::RegAddr::new(self.addr + 0x400 + index * 4) }
}
#[inline]
pub fn r#dio_pad_attr_regwen(self, index: u32) -> register::RegAddr<r#DioPadAttrRegwen> {
assert!(index < 16);
unsafe { register::RegAddr::new(self.addr + 0x4bc + index * 4) }
}
#[inline]
pub fn r#dio_pad_attr(self, index: u32) -> register::RegAddr<r#DioPadAttr> {
assert!(index < 16);
unsafe { register::RegAddr::new(self.addr + 0x4fc + index * 4) }
}
#[inline]
pub fn r#mio_pad_sleep_status(self, index: u32) -> register::RegAddr<r#MioPadSleepStatus> {
assert!(index < 2);
unsafe { register::RegAddr::new(self.addr + 0x53c + index * 4) }
}
#[inline]
pub fn r#mio_pad_sleep_regwen(self, index: u32) -> register::RegAddr<r#MioPadSleepRegwen> {
assert!(index < 47);
unsafe { register::RegAddr::new(self.addr + 0x544 + index * 4) }
}
#[inline]
pub fn r#mio_pad_sleep_en(self, index: u32) -> register::RegAddr<r#MioPadSleepEn> {
assert!(index < 47);
unsafe { register::RegAddr::new(self.addr + 0x600 + index * 4) }
}
#[inline]
pub fn r#mio_pad_sleep_mode(self, index: u32) -> register::RegAddr<r#MioPadSleepMode> {
assert!(index < 47);
unsafe { register::RegAddr::new(self.addr + 0x6bc + index * 4) }
}
#[inline]
pub fn r#dio_pad_sleep_status(self) -> register::RegAddr<r#DioPadSleepStatus> {
unsafe { register::RegAddr::new(self.addr + 0x778) }
}
#[inline]
pub fn r#dio_pad_sleep_regwen(self, index: u32) -> register::RegAddr<r#DioPadSleepRegwen> {
assert!(index < 16);
unsafe { register::RegAddr::new(self.addr + 0x77c + index * 4) }
}
#[inline]
pub fn r#dio_pad_sleep_en(self, index: u32) -> register::RegAddr<r#DioPadSleepEn> {
assert!(index < 16);
unsafe { register::RegAddr::new(self.addr + 0x7bc + index * 4) }
}
#[inline]
pub fn r#dio_pad_sleep_mode(self, index: u32) -> register::RegAddr<r#DioPadSleepMode> {
assert!(index < 16);
unsafe { register::RegAddr::new(self.addr + 0x7fc + index * 4) }
}
#[inline]
pub fn r#wkup_detector_regwen(self, index: u32) -> register::RegAddr<r#WkupDetectorRegwen> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x83c + index * 4) }
}
#[inline]
pub fn r#wkup_detector_en(self, index: u32) -> register::RegAddr<r#WkupDetectorEn> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x85c + index * 4) }
}
#[inline]
pub fn r#wkup_detector(self, index: u32) -> register::RegAddr<r#WkupDetector> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x87c + index * 4) }
}
#[inline]
pub fn r#wkup_detector_cnt_th(self, index: u32) -> register::RegAddr<r#WkupDetectorCntTh> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x89c + index * 4) }
}
#[inline]
pub fn r#wkup_detector_padsel(self, index: u32) -> register::RegAddr<r#WkupDetectorPadsel> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x8bc + index * 4) }
}
#[inline]
pub fn r#wkup_cause(self) -> register::RegAddr<r#WkupCause> {
unsafe { register::RegAddr::new(self.addr + 0x8dc) }
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_fault(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MioPeriphInselRegwen {}
impl register::RegSpec for r#MioPeriphInselRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#MioPeriphInselRegwenRead;
type Write = r#MioPeriphInselRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPeriphInselRegwenRead { pub reg: register::RegRead<r#MioPeriphInselRegwen> }
impl r#MioPeriphInselRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPeriphInselRegwenWrite { pub reg: register::RegWrite<r#MioPeriphInselRegwen> }
impl r#MioPeriphInselRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MioPeriphInsel {}
impl register::RegSpec for r#MioPeriphInsel {
const DEFAULT: u32 = 0x0;
type Read = r#MioPeriphInselRead;
type Write = r#MioPeriphInselWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPeriphInselRead { pub reg: register::RegRead<r#MioPeriphInsel> }
impl r#MioPeriphInselRead {
#[inline]
pub fn r#in(self) -> u32 {
self.reg.field(0x3f)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPeriphInselWrite { pub reg: register::RegWrite<r#MioPeriphInsel> }
impl r#MioPeriphInselWrite {
#[inline]
pub fn r#in(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3f, value); self
}
}
pub enum r#MioOutselRegwen {}
impl register::RegSpec for r#MioOutselRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#MioOutselRegwenRead;
type Write = r#MioOutselRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioOutselRegwenRead { pub reg: register::RegRead<r#MioOutselRegwen> }
impl r#MioOutselRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioOutselRegwenWrite { pub reg: register::RegWrite<r#MioOutselRegwen> }
impl r#MioOutselRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MioOutsel {}
impl register::RegSpec for r#MioOutsel {
const DEFAULT: u32 = 0x2;
type Read = r#MioOutselRead;
type Write = r#MioOutselWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioOutselRead { pub reg: register::RegRead<r#MioOutsel> }
impl r#MioOutselRead {
#[inline]
pub fn r#out(self) -> u32 {
self.reg.field(0x7f)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioOutselWrite { pub reg: register::RegWrite<r#MioOutsel> }
impl r#MioOutselWrite {
#[inline]
pub fn r#out(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7f, value); self
}
}
pub enum r#MioPadAttrRegwen {}
impl register::RegSpec for r#MioPadAttrRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#MioPadAttrRegwenRead;
type Write = r#MioPadAttrRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadAttrRegwenRead { pub reg: register::RegRead<r#MioPadAttrRegwen> }
impl r#MioPadAttrRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadAttrRegwenWrite { pub reg: register::RegWrite<r#MioPadAttrRegwen> }
impl r#MioPadAttrRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MioPadAttr {}
impl register::RegSpec for r#MioPadAttr {
const DEFAULT: u32 = 0x0;
type Read = r#MioPadAttrRead;
type Write = r#MioPadAttrWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadAttrRead { pub reg: register::RegRead<r#MioPadAttr> }
impl r#MioPadAttrRead {
#[inline]
pub fn r#invert(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#virtual_od_en(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#pull_en(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#pull_select(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#keeper_en(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#schmitt_en(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#od_en(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#slew_rate(self) -> u32 {
self.reg.field(0x30000)
}
#[inline]
pub fn r#drive_strength(self) -> u32 {
self.reg.field(0xf00000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadAttrWrite { pub reg: register::RegWrite<r#MioPadAttr> }
impl r#MioPadAttrWrite {
#[inline]
pub fn r#invert(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#virtual_od_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#pull_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#pull_select(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#keeper_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#schmitt_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#od_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#slew_rate(&mut self, value: u32) -> &mut Self {
self.reg.field(0x30000, value); self
}
#[inline]
pub fn r#drive_strength(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
}
pub enum r#DioPadAttrRegwen {}
impl register::RegSpec for r#DioPadAttrRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#DioPadAttrRegwenRead;
type Write = r#DioPadAttrRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadAttrRegwenRead { pub reg: register::RegRead<r#DioPadAttrRegwen> }
impl r#DioPadAttrRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadAttrRegwenWrite { pub reg: register::RegWrite<r#DioPadAttrRegwen> }
impl r#DioPadAttrRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#DioPadAttr {}
impl register::RegSpec for r#DioPadAttr {
const DEFAULT: u32 = 0x0;
type Read = r#DioPadAttrRead;
type Write = r#DioPadAttrWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadAttrRead { pub reg: register::RegRead<r#DioPadAttr> }
impl r#DioPadAttrRead {
#[inline]
pub fn r#invert(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#virtual_od_en(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#pull_en(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#pull_select(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#keeper_en(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#schmitt_en(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#od_en(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#slew_rate(self) -> u32 {
self.reg.field(0x30000)
}
#[inline]
pub fn r#drive_strength(self) -> u32 {
self.reg.field(0xf00000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadAttrWrite { pub reg: register::RegWrite<r#DioPadAttr> }
impl r#DioPadAttrWrite {
#[inline]
pub fn r#invert(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#virtual_od_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#pull_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#pull_select(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#keeper_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#schmitt_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#od_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#slew_rate(&mut self, value: u32) -> &mut Self {
self.reg.field(0x30000, value); self
}
#[inline]
pub fn r#drive_strength(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
}
pub enum r#MioPadSleepStatus {}
impl register::RegSpec for r#MioPadSleepStatus {
const DEFAULT: u32 = 0x0;
type Read = r#MioPadSleepStatusRead;
type Write = r#MioPadSleepStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepStatusRead { pub reg: register::RegRead<r#MioPadSleepStatus> }
impl r#MioPadSleepStatusRead {
#[inline]
pub fn r#en(self, index: u8) -> bool {
assert!(index < 32);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepStatusWrite { pub reg: register::RegWrite<r#MioPadSleepStatus> }
impl r#MioPadSleepStatusWrite {
#[inline]
pub fn r#en(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 32);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#MioPadSleepRegwen {}
impl register::RegSpec for r#MioPadSleepRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#MioPadSleepRegwenRead;
type Write = r#MioPadSleepRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepRegwenRead { pub reg: register::RegRead<r#MioPadSleepRegwen> }
impl r#MioPadSleepRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepRegwenWrite { pub reg: register::RegWrite<r#MioPadSleepRegwen> }
impl r#MioPadSleepRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MioPadSleepEn {}
impl register::RegSpec for r#MioPadSleepEn {
const DEFAULT: u32 = 0x0;
type Read = r#MioPadSleepEnRead;
type Write = r#MioPadSleepEnWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepEnRead { pub reg: register::RegRead<r#MioPadSleepEn> }
impl r#MioPadSleepEnRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepEnWrite { pub reg: register::RegWrite<r#MioPadSleepEn> }
impl r#MioPadSleepEnWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#MioPadSleepMode {}
impl register::RegSpec for r#MioPadSleepMode {
const DEFAULT: u32 = 0x2;
type Read = r#MioPadSleepModeRead;
type Write = r#MioPadSleepModeWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepModeRead { pub reg: register::RegRead<r#MioPadSleepMode> }
impl r#MioPadSleepModeRead {
#[inline]
pub fn r#out(self) -> u32 {
self.reg.field(0x3)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MioPadSleepModeWrite { pub reg: register::RegWrite<r#MioPadSleepMode> }
impl r#MioPadSleepModeWrite {
#[inline]
pub fn r#out(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3, value); self
}
}
pub mod r#mio_pad_sleep_mode {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#Out {
r#TieLow = 0x0,
r#TieHigh = 0x1,
r#HighZ = 0x2,
r#Keep = 0x3,
}
}
pub enum r#DioPadSleepStatus {}
impl register::RegSpec for r#DioPadSleepStatus {
const DEFAULT: u32 = 0x0;
type Read = r#DioPadSleepStatusRead;
type Write = r#DioPadSleepStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepStatusRead { pub reg: register::RegRead<r#DioPadSleepStatus> }
impl r#DioPadSleepStatusRead {
#[inline]
pub fn r#en(self, index: u8) -> bool {
assert!(index < 16);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepStatusWrite { pub reg: register::RegWrite<r#DioPadSleepStatus> }
impl r#DioPadSleepStatusWrite {
#[inline]
pub fn r#en(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 16);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#DioPadSleepRegwen {}
impl register::RegSpec for r#DioPadSleepRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#DioPadSleepRegwenRead;
type Write = r#DioPadSleepRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepRegwenRead { pub reg: register::RegRead<r#DioPadSleepRegwen> }
impl r#DioPadSleepRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepRegwenWrite { pub reg: register::RegWrite<r#DioPadSleepRegwen> }
impl r#DioPadSleepRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#DioPadSleepEn {}
impl register::RegSpec for r#DioPadSleepEn {
const DEFAULT: u32 = 0x0;
type Read = r#DioPadSleepEnRead;
type Write = r#DioPadSleepEnWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepEnRead { pub reg: register::RegRead<r#DioPadSleepEn> }
impl r#DioPadSleepEnRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepEnWrite { pub reg: register::RegWrite<r#DioPadSleepEn> }
impl r#DioPadSleepEnWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#DioPadSleepMode {}
impl register::RegSpec for r#DioPadSleepMode {
const DEFAULT: u32 = 0x2;
type Read = r#DioPadSleepModeRead;
type Write = r#DioPadSleepModeWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepModeRead { pub reg: register::RegRead<r#DioPadSleepMode> }
impl r#DioPadSleepModeRead {
#[inline]
pub fn r#out(self) -> u32 {
self.reg.field(0x3)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DioPadSleepModeWrite { pub reg: register::RegWrite<r#DioPadSleepMode> }
impl r#DioPadSleepModeWrite {
#[inline]
pub fn r#out(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3, value); self
}
}
pub mod r#dio_pad_sleep_mode {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#Out {
r#TieLow = 0x0,
r#TieHigh = 0x1,
r#HighZ = 0x2,
r#Keep = 0x3,
}
}
pub enum r#WkupDetectorRegwen {}
impl register::RegSpec for r#WkupDetectorRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#WkupDetectorRegwenRead;
type Write = r#WkupDetectorRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorRegwenRead { pub reg: register::RegRead<r#WkupDetectorRegwen> }
impl r#WkupDetectorRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorRegwenWrite { pub reg: register::RegWrite<r#WkupDetectorRegwen> }
impl r#WkupDetectorRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#WkupDetectorEn {}
impl register::RegSpec for r#WkupDetectorEn {
const DEFAULT: u32 = 0x0;
type Read = r#WkupDetectorEnRead;
type Write = r#WkupDetectorEnWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorEnRead { pub reg: register::RegRead<r#WkupDetectorEn> }
impl r#WkupDetectorEnRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorEnWrite { pub reg: register::RegWrite<r#WkupDetectorEn> }
impl r#WkupDetectorEnWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#WkupDetector {}
impl register::RegSpec for r#WkupDetector {
const DEFAULT: u32 = 0x0;
type Read = r#WkupDetectorRead;
type Write = r#WkupDetectorWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorRead { pub reg: register::RegRead<r#WkupDetector> }
impl r#WkupDetectorRead {
#[inline]
pub fn r#mode(self) -> u32 {
self.reg.field(0x7)
}
#[inline]
pub fn r#filter(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#miodio(self) -> bool {
self.reg.bit(4)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorWrite { pub reg: register::RegWrite<r#WkupDetector> }
impl r#WkupDetectorWrite {
#[inline]
pub fn r#mode(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7, value); self
}
#[inline]
pub fn r#filter(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#miodio(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
}
pub mod r#wkup_detector {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#Mode {
r#Posedge = 0x0,
r#Negedge = 0x1,
r#Edge = 0x2,
r#Timedhigh = 0x3,
r#Timedlow = 0x4,
}
}
pub enum r#WkupDetectorCntTh {}
impl register::RegSpec for r#WkupDetectorCntTh {
const DEFAULT: u32 = 0x0;
type Read = r#WkupDetectorCntThRead;
type Write = r#WkupDetectorCntThWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorCntThRead { pub reg: register::RegRead<r#WkupDetectorCntTh> }
impl r#WkupDetectorCntThRead {
#[inline]
pub fn r#th(self) -> u32 {
self.reg.field(0xff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorCntThWrite { pub reg: register::RegWrite<r#WkupDetectorCntTh> }
impl r#WkupDetectorCntThWrite {
#[inline]
pub fn r#th(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
}
pub enum r#WkupDetectorPadsel {}
impl register::RegSpec for r#WkupDetectorPadsel {
const DEFAULT: u32 = 0x0;
type Read = r#WkupDetectorPadselRead;
type Write = r#WkupDetectorPadselWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorPadselRead { pub reg: register::RegRead<r#WkupDetectorPadsel> }
impl r#WkupDetectorPadselRead {
#[inline]
pub fn r#sel(self) -> u32 {
self.reg.field(0x3f)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupDetectorPadselWrite { pub reg: register::RegWrite<r#WkupDetectorPadsel> }
impl r#WkupDetectorPadselWrite {
#[inline]
pub fn r#sel(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3f, value); self
}
}
pub enum r#WkupCause {}
impl register::RegSpec for r#WkupCause {
const DEFAULT: u32 = 0x0;
type Read = r#WkupCauseRead;
type Write = r#WkupCauseWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupCauseRead { pub reg: register::RegRead<r#WkupCause> }
impl r#WkupCauseRead {
#[inline]
pub fn r#cause(self, index: u8) -> bool {
assert!(index < 8);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WkupCauseWrite { pub reg: register::RegWrite<r#WkupCause> }
impl r#WkupCauseWrite {
#[inline]
pub fn r#cause(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 8);
self.reg.bit(0 + index * 1, value); self
}
}
}
#[rustfmt::skip]
pub mod r#rstmgr {
impl super::r#Rstmgr {
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#reset_req(self) -> register::RegAddr<r#ResetReq> {
unsafe { register::RegAddr::new(self.addr + 0x4) }
}
#[inline]
pub fn r#reset_info(self) -> register::RegAddr<r#ResetInfo> {
unsafe { register::RegAddr::new(self.addr + 0x8) }
}
#[inline]
pub fn r#alert_regwen(self) -> register::RegAddr<r#AlertRegwen> {
unsafe { register::RegAddr::new(self.addr + 0xc) }
}
#[inline]
pub fn r#alert_info_ctrl(self) -> register::RegAddr<r#AlertInfoCtrl> {
unsafe { register::RegAddr::new(self.addr + 0x10) }
}
#[inline]
pub fn r#alert_info_attr(self) -> register::RegAddr<r#AlertInfoAttr> {
unsafe { register::RegAddr::new(self.addr + 0x14) }
}
#[inline]
pub fn r#alert_info(self) -> register::RegAddr<r#AlertInfo> {
unsafe { register::RegAddr::new(self.addr + 0x18) }
}
#[inline]
pub fn r#cpu_regwen(self) -> register::RegAddr<r#CpuRegwen> {
unsafe { register::RegAddr::new(self.addr + 0x1c) }
}
#[inline]
pub fn r#cpu_info_ctrl(self) -> register::RegAddr<r#CpuInfoCtrl> {
unsafe { register::RegAddr::new(self.addr + 0x20) }
}
#[inline]
pub fn r#cpu_info_attr(self) -> register::RegAddr<r#CpuInfoAttr> {
unsafe { register::RegAddr::new(self.addr + 0x24) }
}
#[inline]
pub fn r#cpu_info(self) -> register::RegAddr<r#CpuInfo> {
unsafe { register::RegAddr::new(self.addr + 0x28) }
}
#[inline]
pub fn r#sw_rst_regwen(self, index: u32) -> register::RegAddr<r#SwRstRegwen> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x2c + index * 4) }
}
#[inline]
pub fn r#sw_rst_ctrl_n(self, index: u32) -> register::RegAddr<r#SwRstCtrlN> {
assert!(index < 8);
unsafe { register::RegAddr::new(self.addr + 0x4c + index * 4) }
}
#[inline]
pub fn r#err_code(self) -> register::RegAddr<r#ErrCode> {
unsafe { register::RegAddr::new(self.addr + 0x6c) }
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_fault(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#fatal_cnsty_fault(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#fatal_cnsty_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#ResetReq {}
impl register::RegSpec for r#ResetReq {
const DEFAULT: u32 = 0x9;
type Read = r#ResetReqRead;
type Write = r#ResetReqWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ResetReqRead { pub reg: register::RegRead<r#ResetReq> }
impl r#ResetReqRead {
#[inline]
pub fn r#val(self) -> u32 {
self.reg.field(0xf)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ResetReqWrite { pub reg: register::RegWrite<r#ResetReq> }
impl r#ResetReqWrite {
#[inline]
pub fn r#val(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
}
pub enum r#ResetInfo {}
impl register::RegSpec for r#ResetInfo {
const DEFAULT: u32 = 0x1;
type Read = r#ResetInfoRead;
type Write = r#ResetInfoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ResetInfoRead { pub reg: register::RegRead<r#ResetInfo> }
impl r#ResetInfoRead {
#[inline]
pub fn r#por(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#low_power_exit(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#sw_reset(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#hw_req(self) -> u32 {
self.reg.field(0xf8)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ResetInfoWrite { pub reg: register::RegWrite<r#ResetInfo> }
impl r#ResetInfoWrite {
#[inline]
pub fn r#por(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#low_power_exit(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#sw_reset(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#hw_req(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf8, value); self
}
}
pub enum r#AlertRegwen {}
impl register::RegSpec for r#AlertRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#AlertRegwenRead;
type Write = r#AlertRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertRegwenRead { pub reg: register::RegRead<r#AlertRegwen> }
impl r#AlertRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertRegwenWrite { pub reg: register::RegWrite<r#AlertRegwen> }
impl r#AlertRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#AlertInfoCtrl {}
impl register::RegSpec for r#AlertInfoCtrl {
const DEFAULT: u32 = 0x0;
type Read = r#AlertInfoCtrlRead;
type Write = r#AlertInfoCtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertInfoCtrlRead { pub reg: register::RegRead<r#AlertInfoCtrl> }
impl r#AlertInfoCtrlRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#index(self) -> u32 {
self.reg.field(0xf0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertInfoCtrlWrite { pub reg: register::RegWrite<r#AlertInfoCtrl> }
impl r#AlertInfoCtrlWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#index(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
}
pub enum r#AlertInfoAttr {}
impl register::RegSpec for r#AlertInfoAttr {
const DEFAULT: u32 = 0x0;
type Read = r#AlertInfoAttrRead;
type Write = r#AlertInfoAttrWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertInfoAttrRead { pub reg: register::RegRead<r#AlertInfoAttr> }
impl r#AlertInfoAttrRead {
#[inline]
pub fn r#cnt_avail(self) -> u32 {
self.reg.field(0xf)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertInfoAttrWrite { pub reg: register::RegWrite<r#AlertInfoAttr> }
impl r#AlertInfoAttrWrite {
#[inline]
pub fn r#cnt_avail(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
}
pub enum r#AlertInfo {}
impl register::RegSpec for r#AlertInfo {
const DEFAULT: u32 = 0x0;
type Read = r#AlertInfoRead;
type Write = r#AlertInfoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertInfoRead { pub reg: register::RegRead<r#AlertInfo> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertInfoWrite { pub reg: register::RegWrite<r#AlertInfo> }
pub enum r#CpuRegwen {}
impl register::RegSpec for r#CpuRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#CpuRegwenRead;
type Write = r#CpuRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuRegwenRead { pub reg: register::RegRead<r#CpuRegwen> }
impl r#CpuRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuRegwenWrite { pub reg: register::RegWrite<r#CpuRegwen> }
impl r#CpuRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#CpuInfoCtrl {}
impl register::RegSpec for r#CpuInfoCtrl {
const DEFAULT: u32 = 0x0;
type Read = r#CpuInfoCtrlRead;
type Write = r#CpuInfoCtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuInfoCtrlRead { pub reg: register::RegRead<r#CpuInfoCtrl> }
impl r#CpuInfoCtrlRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#index(self) -> u32 {
self.reg.field(0xf0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuInfoCtrlWrite { pub reg: register::RegWrite<r#CpuInfoCtrl> }
impl r#CpuInfoCtrlWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#index(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0, value); self
}
}
pub enum r#CpuInfoAttr {}
impl register::RegSpec for r#CpuInfoAttr {
const DEFAULT: u32 = 0x0;
type Read = r#CpuInfoAttrRead;
type Write = r#CpuInfoAttrWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuInfoAttrRead { pub reg: register::RegRead<r#CpuInfoAttr> }
impl r#CpuInfoAttrRead {
#[inline]
pub fn r#cnt_avail(self) -> u32 {
self.reg.field(0xf)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuInfoAttrWrite { pub reg: register::RegWrite<r#CpuInfoAttr> }
impl r#CpuInfoAttrWrite {
#[inline]
pub fn r#cnt_avail(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf, value); self
}
}
pub enum r#CpuInfo {}
impl register::RegSpec for r#CpuInfo {
const DEFAULT: u32 = 0x0;
type Read = r#CpuInfoRead;
type Write = r#CpuInfoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuInfoRead { pub reg: register::RegRead<r#CpuInfo> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CpuInfoWrite { pub reg: register::RegWrite<r#CpuInfo> }
pub enum r#SwRstRegwen {}
impl register::RegSpec for r#SwRstRegwen {
const DEFAULT: u32 = 0x1;
type Read = r#SwRstRegwenRead;
type Write = r#SwRstRegwenWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#SwRstRegwenRead { pub reg: register::RegRead<r#SwRstRegwen> }
impl r#SwRstRegwenRead {
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#SwRstRegwenWrite { pub reg: register::RegWrite<r#SwRstRegwen> }
impl r#SwRstRegwenWrite {
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#SwRstCtrlN {}
impl register::RegSpec for r#SwRstCtrlN {
const DEFAULT: u32 = 0x1;
type Read = r#SwRstCtrlNRead;
type Write = r#SwRstCtrlNWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#SwRstCtrlNRead { pub reg: register::RegRead<r#SwRstCtrlN> }
impl r#SwRstCtrlNRead {
#[inline]
pub fn r#val(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#SwRstCtrlNWrite { pub reg: register::RegWrite<r#SwRstCtrlN> }
impl r#SwRstCtrlNWrite {
#[inline]
pub fn r#val(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#ErrCode {}
impl register::RegSpec for r#ErrCode {
const DEFAULT: u32 = 0x0;
type Read = r#ErrCodeRead;
type Write = r#ErrCodeWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrCodeRead { pub reg: register::RegRead<r#ErrCode> }
impl r#ErrCodeRead {
#[inline]
pub fn r#reg_intg_err(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#reset_consistency_err(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#fsm_err(self) -> bool {
self.reg.bit(2)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ErrCodeWrite { pub reg: register::RegWrite<r#ErrCode> }
impl r#ErrCodeWrite {
#[inline]
pub fn r#reg_intg_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#reset_consistency_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#fsm_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
}
}
#[rustfmt::skip]
pub mod r#rv_plic {
impl super::r#RvPlic {
#[inline]
pub fn r#prio(self, index: u32) -> register::RegAddr<r#Prio> {
assert!(index < 185);
unsafe { register::RegAddr::new(self.addr + 0x0 + index * 4) }
}
#[inline]
pub fn r#ip(self, index: u32) -> register::RegAddr<r#Ip> {
assert!(index < 6);
unsafe { register::RegAddr::new(self.addr + 0x1000 + index * 4) }
}
#[inline]
pub fn r#ie0(self, index: u32) -> register::RegAddr<r#Ie0> {
assert!(index < 6);
unsafe { register::RegAddr::new(self.addr + 0x2000 + index * 4) }
}
#[inline]
pub fn r#threshold(self, index: u32) -> register::RegAddr<r#Threshold> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x200000 + index * 0) }
}
#[inline]
pub fn r#cc(self, index: u32) -> register::RegAddr<r#Cc> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x200004 + index * 0) }
}
#[inline]
pub fn r#msip(self, index: u32) -> register::RegAddr<r#Msip> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x4000000 + index * 0) }
}
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0x4004000) }
}
}
pub enum r#Prio {}
impl register::RegSpec for r#Prio {
const DEFAULT: u32 = 0x0;
type Read = r#PrioRead;
type Write = r#PrioWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PrioRead { pub reg: register::RegRead<r#Prio> }
impl r#PrioRead {
#[inline]
pub fn r#prio(self) -> u32 {
self.reg.field(0x3)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PrioWrite { pub reg: register::RegWrite<r#Prio> }
impl r#PrioWrite {
#[inline]
pub fn r#prio(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3, value); self
}
}
pub enum r#Ip {}
impl register::RegSpec for r#Ip {
const DEFAULT: u32 = 0x0;
type Read = r#IpRead;
type Write = r#IpWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IpRead { pub reg: register::RegRead<r#Ip> }
impl r#IpRead {
#[inline]
pub fn r#p(self, index: u8) -> bool {
assert!(index < 32);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IpWrite { pub reg: register::RegWrite<r#Ip> }
impl r#IpWrite {
#[inline]
pub fn r#p(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 32);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#Ie0 {}
impl register::RegSpec for r#Ie0 {
const DEFAULT: u32 = 0x0;
type Read = r#Ie0Read;
type Write = r#Ie0Write;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Ie0Read { pub reg: register::RegRead<r#Ie0> }
impl r#Ie0Read {
#[inline]
pub fn r#e(self, index: u8) -> bool {
assert!(index < 32);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#Ie0Write { pub reg: register::RegWrite<r#Ie0> }
impl r#Ie0Write {
#[inline]
pub fn r#e(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 32);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#Threshold {}
impl register::RegSpec for r#Threshold {
const DEFAULT: u32 = 0x0;
type Read = r#ThresholdRead;
type Write = r#ThresholdWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ThresholdRead { pub reg: register::RegRead<r#Threshold> }
impl r#ThresholdRead {
#[inline]
pub fn r#threshold(self) -> u32 {
self.reg.field(0x3)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ThresholdWrite { pub reg: register::RegWrite<r#Threshold> }
impl r#ThresholdWrite {
#[inline]
pub fn r#threshold(&mut self, value: u32) -> &mut Self {
self.reg.field(0x3, value); self
}
}
pub enum r#Cc {}
impl register::RegSpec for r#Cc {
const DEFAULT: u32 = 0x0;
type Read = r#CcRead;
type Write = r#CcWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CcRead { pub reg: register::RegRead<r#Cc> }
impl r#CcRead {
#[inline]
pub fn r#cc(self) -> u32 {
self.reg.field(0xff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CcWrite { pub reg: register::RegWrite<r#Cc> }
impl r#CcWrite {
#[inline]
pub fn r#cc(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
}
pub enum r#Msip {}
impl register::RegSpec for r#Msip {
const DEFAULT: u32 = 0x0;
type Read = r#MsipRead;
type Write = r#MsipWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsipRead { pub reg: register::RegRead<r#Msip> }
impl r#MsipRead {
#[inline]
pub fn r#msip(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#MsipWrite { pub reg: register::RegWrite<r#Msip> }
impl r#MsipWrite {
#[inline]
pub fn r#msip(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_fault(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
}
#[rustfmt::skip]
pub mod r#rv_timer {
impl super::r#RvTimer {
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#ctrl(self) -> register::RegAddr<r#Ctrl> {
unsafe { register::RegAddr::new(self.addr + 0x4) }
}
#[inline]
pub fn r#intr_enable(self, index: u32) -> register::RegAddr<r#IntrEnable> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x100 + index * 0) }
}
#[inline]
pub fn r#intr_state(self, index: u32) -> register::RegAddr<r#IntrState> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x104 + index * 0) }
}
#[inline]
pub fn r#intr_test(self, index: u32) -> register::RegAddr<r#IntrTest> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x108 + index * 0) }
}
#[inline]
pub fn r#cfg(self, index: u32) -> register::RegAddr<r#Cfg> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x10c + index * 0) }
}
#[inline]
pub fn r#timer_v_lower(self, index: u32) -> register::RegAddr<r#TimerVLower> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x110 + index * 0) }
}
#[inline]
pub fn r#timer_v_upper(self, index: u32) -> register::RegAddr<r#TimerVUpper> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x114 + index * 0) }
}
#[inline]
pub fn r#compare_lower0(self, index: u32) -> register::RegAddr<r#CompareLower0> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x118 + index * 0) }
}
#[inline]
pub fn r#compare_upper0(self, index: u32) -> register::RegAddr<r#CompareUpper0> {
assert!(index < 1);
unsafe { register::RegAddr::new(self.addr + 0x11c + index * 0) }
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_fault(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Ctrl {}
impl register::RegSpec for r#Ctrl {
const DEFAULT: u32 = 0x0;
type Read = r#CtrlRead;
type Write = r#CtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CtrlRead { pub reg: register::RegRead<r#Ctrl> }
impl r#CtrlRead {
#[inline]
pub fn r#active(self, index: u8) -> bool {
assert!(index < 1);
self.reg.bit(0 + index * 0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CtrlWrite { pub reg: register::RegWrite<r#Ctrl> }
impl r#CtrlWrite {
#[inline]
pub fn r#active(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 1);
self.reg.bit(0 + index * 0, value); self
}
}
pub enum r#IntrEnable {}
impl register::RegSpec for r#IntrEnable {
const DEFAULT: u32 = 0x0;
type Read = r#IntrEnableRead;
type Write = r#IntrEnableWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableRead { pub reg: register::RegRead<r#IntrEnable> }
impl r#IntrEnableRead {
#[inline]
pub fn r#ie(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableWrite { pub reg: register::RegWrite<r#IntrEnable> }
impl r#IntrEnableWrite {
#[inline]
pub fn r#ie(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#IntrState {}
impl register::RegSpec for r#IntrState {
const DEFAULT: u32 = 0x0;
type Read = r#IntrStateRead;
type Write = r#IntrStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateRead { pub reg: register::RegRead<r#IntrState> }
impl r#IntrStateRead {
#[inline]
pub fn r#is(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateWrite { pub reg: register::RegWrite<r#IntrState> }
impl r#IntrStateWrite {
#[inline]
pub fn r#is(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#IntrTest {}
impl register::RegSpec for r#IntrTest {
const DEFAULT: u32 = 0x0;
type Read = r#IntrTestRead;
type Write = r#IntrTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestRead { pub reg: register::RegRead<r#IntrTest> }
impl r#IntrTestRead {
#[inline]
pub fn r#t(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestWrite { pub reg: register::RegWrite<r#IntrTest> }
impl r#IntrTestWrite {
#[inline]
pub fn r#t(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Cfg {}
impl register::RegSpec for r#Cfg {
const DEFAULT: u32 = 0x10000;
type Read = r#CfgRead;
type Write = r#CfgWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CfgRead { pub reg: register::RegRead<r#Cfg> }
impl r#CfgRead {
#[inline]
pub fn r#prescale(self) -> u32 {
self.reg.field(0xfff)
}
#[inline]
pub fn r#step(self) -> u32 {
self.reg.field(0xff0000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CfgWrite { pub reg: register::RegWrite<r#Cfg> }
impl r#CfgWrite {
#[inline]
pub fn r#prescale(&mut self, value: u32) -> &mut Self {
self.reg.field(0xfff, value); self
}
#[inline]
pub fn r#step(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff0000, value); self
}
}
pub enum r#TimerVLower {}
impl register::RegSpec for r#TimerVLower {
const DEFAULT: u32 = 0x0;
type Read = r#TimerVLowerRead;
type Write = r#TimerVLowerWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TimerVLowerRead { pub reg: register::RegRead<r#TimerVLower> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TimerVLowerWrite { pub reg: register::RegWrite<r#TimerVLower> }
pub enum r#TimerVUpper {}
impl register::RegSpec for r#TimerVUpper {
const DEFAULT: u32 = 0x0;
type Read = r#TimerVUpperRead;
type Write = r#TimerVUpperWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TimerVUpperRead { pub reg: register::RegRead<r#TimerVUpper> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TimerVUpperWrite { pub reg: register::RegWrite<r#TimerVUpper> }
pub enum r#CompareLower0 {}
impl register::RegSpec for r#CompareLower0 {
const DEFAULT: u32 = 0xffffffff;
type Read = r#CompareLower0Read;
type Write = r#CompareLower0Write;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CompareLower0Read { pub reg: register::RegRead<r#CompareLower0> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CompareLower0Write { pub reg: register::RegWrite<r#CompareLower0> }
pub enum r#CompareUpper0 {}
impl register::RegSpec for r#CompareUpper0 {
const DEFAULT: u32 = 0xffffffff;
type Read = r#CompareUpper0Read;
type Write = r#CompareUpper0Write;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CompareUpper0Read { pub reg: register::RegRead<r#CompareUpper0> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CompareUpper0Write { pub reg: register::RegWrite<r#CompareUpper0> }
}
#[rustfmt::skip]
pub mod r#uart {
impl super::r#Uart {
#[inline]
pub fn r#intr_state(self) -> register::RegAddr<r#IntrState> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#intr_enable(self) -> register::RegAddr<r#IntrEnable> {
unsafe { register::RegAddr::new(self.addr + 0x4) }
}
#[inline]
pub fn r#intr_test(self) -> register::RegAddr<r#IntrTest> {
unsafe { register::RegAddr::new(self.addr + 0x8) }
}
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0xc) }
}
#[inline]
pub fn r#ctrl(self) -> register::RegAddr<r#Ctrl> {
unsafe { register::RegAddr::new(self.addr + 0x10) }
}
#[inline]
pub fn r#status(self) -> register::RegAddr<r#Status> {
unsafe { register::RegAddr::new(self.addr + 0x14) }
}
#[inline]
pub fn r#rdata(self) -> register::RegAddr<r#Rdata> {
unsafe { register::RegAddr::new(self.addr + 0x18) }
}
#[inline]
pub fn r#wdata(self) -> register::RegAddr<r#Wdata> {
unsafe { register::RegAddr::new(self.addr + 0x1c) }
}
#[inline]
pub fn r#fifo_ctrl(self) -> register::RegAddr<r#FifoCtrl> {
unsafe { register::RegAddr::new(self.addr + 0x20) }
}
#[inline]
pub fn r#fifo_status(self) -> register::RegAddr<r#FifoStatus> {
unsafe { register::RegAddr::new(self.addr + 0x24) }
}
#[inline]
pub fn r#ovrd(self) -> register::RegAddr<r#Ovrd> {
unsafe { register::RegAddr::new(self.addr + 0x28) }
}
#[inline]
pub fn r#val(self) -> register::RegAddr<r#Val> {
unsafe { register::RegAddr::new(self.addr + 0x2c) }
}
#[inline]
pub fn r#timeout_ctrl(self) -> register::RegAddr<r#TimeoutCtrl> {
unsafe { register::RegAddr::new(self.addr + 0x30) }
}
}
pub enum r#IntrState {}
impl register::RegSpec for r#IntrState {
const DEFAULT: u32 = 0x0;
type Read = r#IntrStateRead;
type Write = r#IntrStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateRead { pub reg: register::RegRead<r#IntrState> }
impl r#IntrStateRead {
#[inline]
pub fn r#tx_watermark(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#rx_watermark(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#tx_empty(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#rx_overflow(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#rx_frame_err(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#rx_break_err(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#rx_timeout(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#rx_parity_err(self) -> bool {
self.reg.bit(7)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateWrite { pub reg: register::RegWrite<r#IntrState> }
impl r#IntrStateWrite {
#[inline]
pub fn r#tx_watermark(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#rx_watermark(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#tx_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#rx_overflow(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#rx_frame_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#rx_break_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#rx_timeout(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#rx_parity_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
}
pub enum r#IntrEnable {}
impl register::RegSpec for r#IntrEnable {
const DEFAULT: u32 = 0x0;
type Read = r#IntrEnableRead;
type Write = r#IntrEnableWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableRead { pub reg: register::RegRead<r#IntrEnable> }
impl r#IntrEnableRead {
#[inline]
pub fn r#tx_watermark(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#rx_watermark(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#tx_empty(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#rx_overflow(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#rx_frame_err(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#rx_break_err(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#rx_timeout(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#rx_parity_err(self) -> bool {
self.reg.bit(7)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableWrite { pub reg: register::RegWrite<r#IntrEnable> }
impl r#IntrEnableWrite {
#[inline]
pub fn r#tx_watermark(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#rx_watermark(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#tx_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#rx_overflow(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#rx_frame_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#rx_break_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#rx_timeout(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#rx_parity_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
}
pub enum r#IntrTest {}
impl register::RegSpec for r#IntrTest {
const DEFAULT: u32 = 0x0;
type Read = r#IntrTestRead;
type Write = r#IntrTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestRead { pub reg: register::RegRead<r#IntrTest> }
impl r#IntrTestRead {
#[inline]
pub fn r#tx_watermark(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#rx_watermark(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#tx_empty(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#rx_overflow(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#rx_frame_err(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#rx_break_err(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#rx_timeout(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#rx_parity_err(self) -> bool {
self.reg.bit(7)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestWrite { pub reg: register::RegWrite<r#IntrTest> }
impl r#IntrTestWrite {
#[inline]
pub fn r#tx_watermark(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#rx_watermark(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#tx_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#rx_overflow(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#rx_frame_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#rx_break_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#rx_timeout(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#rx_parity_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_fault(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Ctrl {}
impl register::RegSpec for r#Ctrl {
const DEFAULT: u32 = 0x0;
type Read = r#CtrlRead;
type Write = r#CtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CtrlRead { pub reg: register::RegRead<r#Ctrl> }
impl r#CtrlRead {
#[inline]
pub fn r#tx(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#rx(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#nf(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#slpbk(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#llpbk(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#parity_en(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#parity_odd(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#rxblvl(self) -> u32 {
self.reg.field(0x300)
}
#[inline]
pub fn r#nco(self) -> u32 {
self.reg.field(0xffff0000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#CtrlWrite { pub reg: register::RegWrite<r#Ctrl> }
impl r#CtrlWrite {
#[inline]
pub fn r#tx(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#rx(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#nf(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#slpbk(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#llpbk(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#parity_en(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#parity_odd(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#rxblvl(&mut self, value: u32) -> &mut Self {
self.reg.field(0x300, value); self
}
#[inline]
pub fn r#nco(&mut self, value: u32) -> &mut Self {
self.reg.field(0xffff0000, value); self
}
}
pub mod r#ctrl {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#Rxblvl {
r#Break2 = 0x0,
r#Break4 = 0x1,
r#Break8 = 0x2,
r#Break16 = 0x3,
}
}
pub enum r#Status {}
impl register::RegSpec for r#Status {
const DEFAULT: u32 = 0x3c;
type Read = r#StatusRead;
type Write = r#StatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusRead { pub reg: register::RegRead<r#Status> }
impl r#StatusRead {
#[inline]
pub fn r#txfull(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#rxfull(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#txempty(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#txidle(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#rxidle(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#rxempty(self) -> bool {
self.reg.bit(5)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#StatusWrite { pub reg: register::RegWrite<r#Status> }
impl r#StatusWrite {
#[inline]
pub fn r#txfull(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#rxfull(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#txempty(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#txidle(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#rxidle(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#rxempty(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
}
pub enum r#Rdata {}
impl register::RegSpec for r#Rdata {
const DEFAULT: u32 = 0x0;
type Read = r#RdataRead;
type Write = r#RdataWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RdataRead { pub reg: register::RegRead<r#Rdata> }
impl r#RdataRead {
#[inline]
pub fn r#rdata(self) -> u32 {
self.reg.field(0xff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RdataWrite { pub reg: register::RegWrite<r#Rdata> }
impl r#RdataWrite {
#[inline]
pub fn r#rdata(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
}
pub enum r#Wdata {}
impl register::RegSpec for r#Wdata {
const DEFAULT: u32 = 0x0;
type Read = r#WdataRead;
type Write = r#WdataWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WdataRead { pub reg: register::RegRead<r#Wdata> }
impl r#WdataRead {
#[inline]
pub fn r#wdata(self) -> u32 {
self.reg.field(0xff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WdataWrite { pub reg: register::RegWrite<r#Wdata> }
impl r#WdataWrite {
#[inline]
pub fn r#wdata(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
}
pub enum r#FifoCtrl {}
impl register::RegSpec for r#FifoCtrl {
const DEFAULT: u32 = 0x0;
type Read = r#FifoCtrlRead;
type Write = r#FifoCtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoCtrlRead { pub reg: register::RegRead<r#FifoCtrl> }
impl r#FifoCtrlRead {
#[inline]
pub fn r#rxrst(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#txrst(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#rxilvl(self) -> u32 {
self.reg.field(0x1c)
}
#[inline]
pub fn r#txilvl(self) -> u32 {
self.reg.field(0xe0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoCtrlWrite { pub reg: register::RegWrite<r#FifoCtrl> }
impl r#FifoCtrlWrite {
#[inline]
pub fn r#rxrst(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#txrst(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#rxilvl(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1c, value); self
}
#[inline]
pub fn r#txilvl(&mut self, value: u32) -> &mut Self {
self.reg.field(0xe0, value); self
}
}
pub mod r#fifo_ctrl {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#Rxilvl {
r#Rxlvl1 = 0x0,
r#Rxlvl2 = 0x1,
r#Rxlvl4 = 0x2,
r#Rxlvl8 = 0x3,
r#Rxlvl16 = 0x4,
r#Rxlvl32 = 0x5,
r#Rxlvl64 = 0x6,
r#Rxlvl126 = 0x7,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#Txilvl {
r#Txlvl1 = 0x0,
r#Txlvl2 = 0x1,
r#Txlvl4 = 0x2,
r#Txlvl8 = 0x3,
r#Txlvl16 = 0x4,
r#Txlvl32 = 0x5,
r#Txlvl64 = 0x6,
}
}
pub enum r#FifoStatus {}
impl register::RegSpec for r#FifoStatus {
const DEFAULT: u32 = 0x0;
type Read = r#FifoStatusRead;
type Write = r#FifoStatusWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoStatusRead { pub reg: register::RegRead<r#FifoStatus> }
impl r#FifoStatusRead {
#[inline]
pub fn r#txlvl(self) -> u32 {
self.reg.field(0xff)
}
#[inline]
pub fn r#rxlvl(self) -> u32 {
self.reg.field(0xff0000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#FifoStatusWrite { pub reg: register::RegWrite<r#FifoStatus> }
impl r#FifoStatusWrite {
#[inline]
pub fn r#txlvl(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff, value); self
}
#[inline]
pub fn r#rxlvl(&mut self, value: u32) -> &mut Self {
self.reg.field(0xff0000, value); self
}
}
pub enum r#Ovrd {}
impl register::RegSpec for r#Ovrd {
const DEFAULT: u32 = 0x0;
type Read = r#OvrdRead;
type Write = r#OvrdWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OvrdRead { pub reg: register::RegRead<r#Ovrd> }
impl r#OvrdRead {
#[inline]
pub fn r#txen(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#txval(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OvrdWrite { pub reg: register::RegWrite<r#Ovrd> }
impl r#OvrdWrite {
#[inline]
pub fn r#txen(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#txval(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#Val {}
impl register::RegSpec for r#Val {
const DEFAULT: u32 = 0x0;
type Read = r#ValRead;
type Write = r#ValWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ValRead { pub reg: register::RegRead<r#Val> }
impl r#ValRead {
#[inline]
pub fn r#rx(self) -> u32 {
self.reg.field(0xffff)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ValWrite { pub reg: register::RegWrite<r#Val> }
impl r#ValWrite {
#[inline]
pub fn r#rx(&mut self, value: u32) -> &mut Self {
self.reg.field(0xffff, value); self
}
}
pub enum r#TimeoutCtrl {}
impl register::RegSpec for r#TimeoutCtrl {
const DEFAULT: u32 = 0x0;
type Read = r#TimeoutCtrlRead;
type Write = r#TimeoutCtrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TimeoutCtrlRead { pub reg: register::RegRead<r#TimeoutCtrl> }
impl r#TimeoutCtrlRead {
#[inline]
pub fn r#val(self) -> u32 {
self.reg.field(0xffffff)
}
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(31)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#TimeoutCtrlWrite { pub reg: register::RegWrite<r#TimeoutCtrl> }
impl r#TimeoutCtrlWrite {
#[inline]
pub fn r#val(&mut self, value: u32) -> &mut Self {
self.reg.field(0xffffff, value); self
}
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(31, value); self
}
}
}
#[rustfmt::skip]
pub mod r#usbdev {
impl super::r#Usbdev {
#[inline]
pub fn r#intr_state(self) -> register::RegAddr<r#IntrState> {
unsafe { register::RegAddr::new(self.addr + 0x0) }
}
#[inline]
pub fn r#intr_enable(self) -> register::RegAddr<r#IntrEnable> {
unsafe { register::RegAddr::new(self.addr + 0x4) }
}
#[inline]
pub fn r#intr_test(self) -> register::RegAddr<r#IntrTest> {
unsafe { register::RegAddr::new(self.addr + 0x8) }
}
#[inline]
pub fn r#alert_test(self) -> register::RegAddr<r#AlertTest> {
unsafe { register::RegAddr::new(self.addr + 0xc) }
}
#[inline]
pub fn r#usbctrl(self) -> register::RegAddr<r#Usbctrl> {
unsafe { register::RegAddr::new(self.addr + 0x10) }
}
#[inline]
pub fn r#ep_out_enable(self) -> register::RegAddr<r#EpOutEnable> {
unsafe { register::RegAddr::new(self.addr + 0x14) }
}
#[inline]
pub fn r#ep_in_enable(self) -> register::RegAddr<r#EpInEnable> {
unsafe { register::RegAddr::new(self.addr + 0x18) }
}
#[inline]
pub fn r#usbstat(self) -> register::RegAddr<r#Usbstat> {
unsafe { register::RegAddr::new(self.addr + 0x1c) }
}
#[inline]
pub fn r#avbuffer(self) -> register::RegAddr<r#Avbuffer> {
unsafe { register::RegAddr::new(self.addr + 0x20) }
}
#[inline]
pub fn r#rxfifo(self) -> register::RegAddr<r#Rxfifo> {
unsafe { register::RegAddr::new(self.addr + 0x24) }
}
#[inline]
pub fn r#rxenable_setup(self) -> register::RegAddr<r#RxenableSetup> {
unsafe { register::RegAddr::new(self.addr + 0x28) }
}
#[inline]
pub fn r#rxenable_out(self) -> register::RegAddr<r#RxenableOut> {
unsafe { register::RegAddr::new(self.addr + 0x2c) }
}
#[inline]
pub fn r#set_nak_out(self) -> register::RegAddr<r#SetNakOut> {
unsafe { register::RegAddr::new(self.addr + 0x30) }
}
#[inline]
pub fn r#in_sent(self) -> register::RegAddr<r#InSent> {
unsafe { register::RegAddr::new(self.addr + 0x34) }
}
#[inline]
pub fn r#out_stall(self) -> register::RegAddr<r#OutStall> {
unsafe { register::RegAddr::new(self.addr + 0x38) }
}
#[inline]
pub fn r#in_stall(self) -> register::RegAddr<r#InStall> {
unsafe { register::RegAddr::new(self.addr + 0x3c) }
}
#[inline]
pub fn r#configin(self, index: u32) -> register::RegAddr<r#Configin> {
assert!(index < 12);
unsafe { register::RegAddr::new(self.addr + 0x40 + index * 4) }
}
#[inline]
pub fn r#out_iso(self) -> register::RegAddr<r#OutIso> {
unsafe { register::RegAddr::new(self.addr + 0x70) }
}
#[inline]
pub fn r#in_iso(self) -> register::RegAddr<r#InIso> {
unsafe { register::RegAddr::new(self.addr + 0x74) }
}
#[inline]
pub fn r#data_toggle_clear(self) -> register::RegAddr<r#DataToggleClear> {
unsafe { register::RegAddr::new(self.addr + 0x78) }
}
#[inline]
pub fn r#phy_pins_sense(self) -> register::RegAddr<r#PhyPinsSense> {
unsafe { register::RegAddr::new(self.addr + 0x7c) }
}
#[inline]
pub fn r#phy_pins_drive(self) -> register::RegAddr<r#PhyPinsDrive> {
unsafe { register::RegAddr::new(self.addr + 0x80) }
}
#[inline]
pub fn r#phy_config(self) -> register::RegAddr<r#PhyConfig> {
unsafe { register::RegAddr::new(self.addr + 0x84) }
}
#[inline]
pub fn r#wake_control(self) -> register::RegAddr<r#WakeControl> {
unsafe { register::RegAddr::new(self.addr + 0x88) }
}
#[inline]
pub fn r#wake_events(self) -> register::RegAddr<r#WakeEvents> {
unsafe { register::RegAddr::new(self.addr + 0x8c) }
}
#[inline]
pub fn r#buffer(self, index: u32) -> register::RegAddr<r#Buffer> {
assert!(index < 512);
unsafe { register::RegAddr::new(self.addr + 0x800 + index * 4) }
}
}
pub enum r#IntrState {}
impl register::RegSpec for r#IntrState {
const DEFAULT: u32 = 0x0;
type Read = r#IntrStateRead;
type Write = r#IntrStateWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateRead { pub reg: register::RegRead<r#IntrState> }
impl r#IntrStateRead {
#[inline]
pub fn r#pkt_received(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#pkt_sent(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#disconnected(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#host_lost(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#link_reset(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#link_suspend(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#link_resume(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#av_empty(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#rx_full(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#av_overflow(self) -> bool {
self.reg.bit(9)
}
#[inline]
pub fn r#link_in_err(self) -> bool {
self.reg.bit(10)
}
#[inline]
pub fn r#rx_crc_err(self) -> bool {
self.reg.bit(11)
}
#[inline]
pub fn r#rx_pid_err(self) -> bool {
self.reg.bit(12)
}
#[inline]
pub fn r#rx_bitstuff_err(self) -> bool {
self.reg.bit(13)
}
#[inline]
pub fn r#frame(self) -> bool {
self.reg.bit(14)
}
#[inline]
pub fn r#powered(self) -> bool {
self.reg.bit(15)
}
#[inline]
pub fn r#link_out_err(self) -> bool {
self.reg.bit(16)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrStateWrite { pub reg: register::RegWrite<r#IntrState> }
impl r#IntrStateWrite {
#[inline]
pub fn r#pkt_received(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#pkt_sent(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#disconnected(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#host_lost(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#link_reset(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#link_suspend(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#link_resume(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#av_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#rx_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#av_overflow(&mut self, value: bool) -> &mut Self {
self.reg.bit(9, value); self
}
#[inline]
pub fn r#link_in_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(10, value); self
}
#[inline]
pub fn r#rx_crc_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(11, value); self
}
#[inline]
pub fn r#rx_pid_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(12, value); self
}
#[inline]
pub fn r#rx_bitstuff_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(13, value); self
}
#[inline]
pub fn r#frame(&mut self, value: bool) -> &mut Self {
self.reg.bit(14, value); self
}
#[inline]
pub fn r#powered(&mut self, value: bool) -> &mut Self {
self.reg.bit(15, value); self
}
#[inline]
pub fn r#link_out_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(16, value); self
}
}
pub enum r#IntrEnable {}
impl register::RegSpec for r#IntrEnable {
const DEFAULT: u32 = 0x0;
type Read = r#IntrEnableRead;
type Write = r#IntrEnableWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableRead { pub reg: register::RegRead<r#IntrEnable> }
impl r#IntrEnableRead {
#[inline]
pub fn r#pkt_received(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#pkt_sent(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#disconnected(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#host_lost(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#link_reset(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#link_suspend(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#link_resume(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#av_empty(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#rx_full(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#av_overflow(self) -> bool {
self.reg.bit(9)
}
#[inline]
pub fn r#link_in_err(self) -> bool {
self.reg.bit(10)
}
#[inline]
pub fn r#rx_crc_err(self) -> bool {
self.reg.bit(11)
}
#[inline]
pub fn r#rx_pid_err(self) -> bool {
self.reg.bit(12)
}
#[inline]
pub fn r#rx_bitstuff_err(self) -> bool {
self.reg.bit(13)
}
#[inline]
pub fn r#frame(self) -> bool {
self.reg.bit(14)
}
#[inline]
pub fn r#powered(self) -> bool {
self.reg.bit(15)
}
#[inline]
pub fn r#link_out_err(self) -> bool {
self.reg.bit(16)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrEnableWrite { pub reg: register::RegWrite<r#IntrEnable> }
impl r#IntrEnableWrite {
#[inline]
pub fn r#pkt_received(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#pkt_sent(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#disconnected(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#host_lost(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#link_reset(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#link_suspend(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#link_resume(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#av_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#rx_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#av_overflow(&mut self, value: bool) -> &mut Self {
self.reg.bit(9, value); self
}
#[inline]
pub fn r#link_in_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(10, value); self
}
#[inline]
pub fn r#rx_crc_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(11, value); self
}
#[inline]
pub fn r#rx_pid_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(12, value); self
}
#[inline]
pub fn r#rx_bitstuff_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(13, value); self
}
#[inline]
pub fn r#frame(&mut self, value: bool) -> &mut Self {
self.reg.bit(14, value); self
}
#[inline]
pub fn r#powered(&mut self, value: bool) -> &mut Self {
self.reg.bit(15, value); self
}
#[inline]
pub fn r#link_out_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(16, value); self
}
}
pub enum r#IntrTest {}
impl register::RegSpec for r#IntrTest {
const DEFAULT: u32 = 0x0;
type Read = r#IntrTestRead;
type Write = r#IntrTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestRead { pub reg: register::RegRead<r#IntrTest> }
impl r#IntrTestRead {
#[inline]
pub fn r#pkt_received(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#pkt_sent(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#disconnected(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#host_lost(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#link_reset(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#link_suspend(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#link_resume(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#av_empty(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#rx_full(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#av_overflow(self) -> bool {
self.reg.bit(9)
}
#[inline]
pub fn r#link_in_err(self) -> bool {
self.reg.bit(10)
}
#[inline]
pub fn r#rx_crc_err(self) -> bool {
self.reg.bit(11)
}
#[inline]
pub fn r#rx_pid_err(self) -> bool {
self.reg.bit(12)
}
#[inline]
pub fn r#rx_bitstuff_err(self) -> bool {
self.reg.bit(13)
}
#[inline]
pub fn r#frame(self) -> bool {
self.reg.bit(14)
}
#[inline]
pub fn r#powered(self) -> bool {
self.reg.bit(15)
}
#[inline]
pub fn r#link_out_err(self) -> bool {
self.reg.bit(16)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#IntrTestWrite { pub reg: register::RegWrite<r#IntrTest> }
impl r#IntrTestWrite {
#[inline]
pub fn r#pkt_received(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#pkt_sent(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#disconnected(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#host_lost(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#link_reset(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#link_suspend(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#link_resume(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#av_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#rx_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#av_overflow(&mut self, value: bool) -> &mut Self {
self.reg.bit(9, value); self
}
#[inline]
pub fn r#link_in_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(10, value); self
}
#[inline]
pub fn r#rx_crc_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(11, value); self
}
#[inline]
pub fn r#rx_pid_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(12, value); self
}
#[inline]
pub fn r#rx_bitstuff_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(13, value); self
}
#[inline]
pub fn r#frame(&mut self, value: bool) -> &mut Self {
self.reg.bit(14, value); self
}
#[inline]
pub fn r#powered(&mut self, value: bool) -> &mut Self {
self.reg.bit(15, value); self
}
#[inline]
pub fn r#link_out_err(&mut self, value: bool) -> &mut Self {
self.reg.bit(16, value); self
}
}
pub enum r#AlertTest {}
impl register::RegSpec for r#AlertTest {
const DEFAULT: u32 = 0x0;
type Read = r#AlertTestRead;
type Write = r#AlertTestWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestRead { pub reg: register::RegRead<r#AlertTest> }
impl r#AlertTestRead {
#[inline]
pub fn r#fatal_fault(self) -> bool {
self.reg.bit(0)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AlertTestWrite { pub reg: register::RegWrite<r#AlertTest> }
impl r#AlertTestWrite {
#[inline]
pub fn r#fatal_fault(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
}
pub enum r#Usbctrl {}
impl register::RegSpec for r#Usbctrl {
const DEFAULT: u32 = 0x0;
type Read = r#UsbctrlRead;
type Write = r#UsbctrlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#UsbctrlRead { pub reg: register::RegRead<r#Usbctrl> }
impl r#UsbctrlRead {
#[inline]
pub fn r#enable(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#resume_link_active(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#device_address(self) -> u32 {
self.reg.field(0x7f0000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#UsbctrlWrite { pub reg: register::RegWrite<r#Usbctrl> }
impl r#UsbctrlWrite {
#[inline]
pub fn r#enable(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#resume_link_active(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#device_address(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7f0000, value); self
}
}
pub enum r#EpOutEnable {}
impl register::RegSpec for r#EpOutEnable {
const DEFAULT: u32 = 0x0;
type Read = r#EpOutEnableRead;
type Write = r#EpOutEnableWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EpOutEnableRead { pub reg: register::RegRead<r#EpOutEnable> }
impl r#EpOutEnableRead {
#[inline]
pub fn r#enable(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EpOutEnableWrite { pub reg: register::RegWrite<r#EpOutEnable> }
impl r#EpOutEnableWrite {
#[inline]
pub fn r#enable(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#EpInEnable {}
impl register::RegSpec for r#EpInEnable {
const DEFAULT: u32 = 0x0;
type Read = r#EpInEnableRead;
type Write = r#EpInEnableWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EpInEnableRead { pub reg: register::RegRead<r#EpInEnable> }
impl r#EpInEnableRead {
#[inline]
pub fn r#enable(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#EpInEnableWrite { pub reg: register::RegWrite<r#EpInEnable> }
impl r#EpInEnableWrite {
#[inline]
pub fn r#enable(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#Usbstat {}
impl register::RegSpec for r#Usbstat {
const DEFAULT: u32 = 0x80000000;
type Read = r#UsbstatRead;
type Write = r#UsbstatWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#UsbstatRead { pub reg: register::RegRead<r#Usbstat> }
impl r#UsbstatRead {
#[inline]
pub fn r#frame(self) -> u32 {
self.reg.field(0x7ff)
}
#[inline]
pub fn r#host_lost(self) -> bool {
self.reg.bit(11)
}
#[inline]
pub fn r#link_state(self) -> u32 {
self.reg.field(0x7000)
}
#[inline]
pub fn r#sense(self) -> bool {
self.reg.bit(15)
}
#[inline]
pub fn r#av_depth(self) -> u32 {
self.reg.field(0xf0000)
}
#[inline]
pub fn r#av_full(self) -> bool {
self.reg.bit(23)
}
#[inline]
pub fn r#rx_depth(self) -> u32 {
self.reg.field(0xf000000)
}
#[inline]
pub fn r#rx_empty(self) -> bool {
self.reg.bit(31)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#UsbstatWrite { pub reg: register::RegWrite<r#Usbstat> }
impl r#UsbstatWrite {
#[inline]
pub fn r#frame(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7ff, value); self
}
#[inline]
pub fn r#host_lost(&mut self, value: bool) -> &mut Self {
self.reg.bit(11, value); self
}
#[inline]
pub fn r#link_state(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7000, value); self
}
#[inline]
pub fn r#sense(&mut self, value: bool) -> &mut Self {
self.reg.bit(15, value); self
}
#[inline]
pub fn r#av_depth(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf0000, value); self
}
#[inline]
pub fn r#av_full(&mut self, value: bool) -> &mut Self {
self.reg.bit(23, value); self
}
#[inline]
pub fn r#rx_depth(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf000000, value); self
}
#[inline]
pub fn r#rx_empty(&mut self, value: bool) -> &mut Self {
self.reg.bit(31, value); self
}
}
pub mod r#usbstat {
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum r#LinkState {
r#Disconnected = 0x0,
r#Powered = 0x1,
r#PoweredSuspended = 0x2,
r#Active = 0x3,
r#Suspended = 0x4,
r#ActiveNosof = 0x5,
r#Resuming = 0x6,
}
}
pub enum r#Avbuffer {}
impl register::RegSpec for r#Avbuffer {
const DEFAULT: u32 = 0x0;
type Read = r#AvbufferRead;
type Write = r#AvbufferWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AvbufferRead { pub reg: register::RegRead<r#Avbuffer> }
impl r#AvbufferRead {
#[inline]
pub fn r#buffer(self) -> u32 {
self.reg.field(0x1f)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#AvbufferWrite { pub reg: register::RegWrite<r#Avbuffer> }
impl r#AvbufferWrite {
#[inline]
pub fn r#buffer(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f, value); self
}
}
pub enum r#Rxfifo {}
impl register::RegSpec for r#Rxfifo {
const DEFAULT: u32 = 0x0;
type Read = r#RxfifoRead;
type Write = r#RxfifoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RxfifoRead { pub reg: register::RegRead<r#Rxfifo> }
impl r#RxfifoRead {
#[inline]
pub fn r#buffer(self) -> u32 {
self.reg.field(0x1f)
}
#[inline]
pub fn r#size(self) -> u32 {
self.reg.field(0x7f00)
}
#[inline]
pub fn r#setup(self) -> bool {
self.reg.bit(19)
}
#[inline]
pub fn r#ep(self) -> u32 {
self.reg.field(0xf00000)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RxfifoWrite { pub reg: register::RegWrite<r#Rxfifo> }
impl r#RxfifoWrite {
#[inline]
pub fn r#buffer(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f, value); self
}
#[inline]
pub fn r#size(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7f00, value); self
}
#[inline]
pub fn r#setup(&mut self, value: bool) -> &mut Self {
self.reg.bit(19, value); self
}
#[inline]
pub fn r#ep(&mut self, value: u32) -> &mut Self {
self.reg.field(0xf00000, value); self
}
}
pub enum r#RxenableSetup {}
impl register::RegSpec for r#RxenableSetup {
const DEFAULT: u32 = 0x0;
type Read = r#RxenableSetupRead;
type Write = r#RxenableSetupWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RxenableSetupRead { pub reg: register::RegRead<r#RxenableSetup> }
impl r#RxenableSetupRead {
#[inline]
pub fn r#setup(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RxenableSetupWrite { pub reg: register::RegWrite<r#RxenableSetup> }
impl r#RxenableSetupWrite {
#[inline]
pub fn r#setup(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#RxenableOut {}
impl register::RegSpec for r#RxenableOut {
const DEFAULT: u32 = 0x0;
type Read = r#RxenableOutRead;
type Write = r#RxenableOutWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RxenableOutRead { pub reg: register::RegRead<r#RxenableOut> }
impl r#RxenableOutRead {
#[inline]
pub fn r#out(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#RxenableOutWrite { pub reg: register::RegWrite<r#RxenableOut> }
impl r#RxenableOutWrite {
#[inline]
pub fn r#out(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#SetNakOut {}
impl register::RegSpec for r#SetNakOut {
const DEFAULT: u32 = 0x0;
type Read = r#SetNakOutRead;
type Write = r#SetNakOutWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#SetNakOutRead { pub reg: register::RegRead<r#SetNakOut> }
impl r#SetNakOutRead {
#[inline]
pub fn r#enable(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#SetNakOutWrite { pub reg: register::RegWrite<r#SetNakOut> }
impl r#SetNakOutWrite {
#[inline]
pub fn r#enable(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#InSent {}
impl register::RegSpec for r#InSent {
const DEFAULT: u32 = 0x0;
type Read = r#InSentRead;
type Write = r#InSentWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InSentRead { pub reg: register::RegRead<r#InSent> }
impl r#InSentRead {
#[inline]
pub fn r#sent(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InSentWrite { pub reg: register::RegWrite<r#InSent> }
impl r#InSentWrite {
#[inline]
pub fn r#sent(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#OutStall {}
impl register::RegSpec for r#OutStall {
const DEFAULT: u32 = 0x0;
type Read = r#OutStallRead;
type Write = r#OutStallWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OutStallRead { pub reg: register::RegRead<r#OutStall> }
impl r#OutStallRead {
#[inline]
pub fn r#endpoint(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OutStallWrite { pub reg: register::RegWrite<r#OutStall> }
impl r#OutStallWrite {
#[inline]
pub fn r#endpoint(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#InStall {}
impl register::RegSpec for r#InStall {
const DEFAULT: u32 = 0x0;
type Read = r#InStallRead;
type Write = r#InStallWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InStallRead { pub reg: register::RegRead<r#InStall> }
impl r#InStallRead {
#[inline]
pub fn r#endpoint(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InStallWrite { pub reg: register::RegWrite<r#InStall> }
impl r#InStallWrite {
#[inline]
pub fn r#endpoint(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#Configin {}
impl register::RegSpec for r#Configin {
const DEFAULT: u32 = 0x0;
type Read = r#ConfiginRead;
type Write = r#ConfiginWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ConfiginRead { pub reg: register::RegRead<r#Configin> }
impl r#ConfiginRead {
#[inline]
pub fn r#buffer(self) -> u32 {
self.reg.field(0x1f)
}
#[inline]
pub fn r#size(self) -> u32 {
self.reg.field(0x7f00)
}
#[inline]
pub fn r#pend(self) -> bool {
self.reg.bit(30)
}
#[inline]
pub fn r#rdy(self) -> bool {
self.reg.bit(31)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#ConfiginWrite { pub reg: register::RegWrite<r#Configin> }
impl r#ConfiginWrite {
#[inline]
pub fn r#buffer(&mut self, value: u32) -> &mut Self {
self.reg.field(0x1f, value); self
}
#[inline]
pub fn r#size(&mut self, value: u32) -> &mut Self {
self.reg.field(0x7f00, value); self
}
#[inline]
pub fn r#pend(&mut self, value: bool) -> &mut Self {
self.reg.bit(30, value); self
}
#[inline]
pub fn r#rdy(&mut self, value: bool) -> &mut Self {
self.reg.bit(31, value); self
}
}
pub enum r#OutIso {}
impl register::RegSpec for r#OutIso {
const DEFAULT: u32 = 0x0;
type Read = r#OutIsoRead;
type Write = r#OutIsoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OutIsoRead { pub reg: register::RegRead<r#OutIso> }
impl r#OutIsoRead {
#[inline]
pub fn r#iso(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#OutIsoWrite { pub reg: register::RegWrite<r#OutIso> }
impl r#OutIsoWrite {
#[inline]
pub fn r#iso(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#InIso {}
impl register::RegSpec for r#InIso {
const DEFAULT: u32 = 0x0;
type Read = r#InIsoRead;
type Write = r#InIsoWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InIsoRead { pub reg: register::RegRead<r#InIso> }
impl r#InIsoRead {
#[inline]
pub fn r#iso(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#InIsoWrite { pub reg: register::RegWrite<r#InIso> }
impl r#InIsoWrite {
#[inline]
pub fn r#iso(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#DataToggleClear {}
impl register::RegSpec for r#DataToggleClear {
const DEFAULT: u32 = 0x0;
type Read = r#DataToggleClearRead;
type Write = r#DataToggleClearWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DataToggleClearRead { pub reg: register::RegRead<r#DataToggleClear> }
impl r#DataToggleClearRead {
#[inline]
pub fn r#clear(self, index: u8) -> bool {
assert!(index < 12);
self.reg.bit(0 + index * 1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#DataToggleClearWrite { pub reg: register::RegWrite<r#DataToggleClear> }
impl r#DataToggleClearWrite {
#[inline]
pub fn r#clear(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 12);
self.reg.bit(0 + index * 1, value); self
}
}
pub enum r#PhyPinsSense {}
impl register::RegSpec for r#PhyPinsSense {
const DEFAULT: u32 = 0x0;
type Read = r#PhyPinsSenseRead;
type Write = r#PhyPinsSenseWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyPinsSenseRead { pub reg: register::RegRead<r#PhyPinsSense> }
impl r#PhyPinsSenseRead {
#[inline]
pub fn r#rx_dp_i(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#rx_dn_i(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#rx_d_i(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#tx_dp_o(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#tx_dn_o(self) -> bool {
self.reg.bit(9)
}
#[inline]
pub fn r#tx_d_o(self) -> bool {
self.reg.bit(10)
}
#[inline]
pub fn r#tx_se0_o(self) -> bool {
self.reg.bit(11)
}
#[inline]
pub fn r#tx_oe_o(self) -> bool {
self.reg.bit(12)
}
#[inline]
pub fn r#pwr_sense(self) -> bool {
self.reg.bit(16)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyPinsSenseWrite { pub reg: register::RegWrite<r#PhyPinsSense> }
impl r#PhyPinsSenseWrite {
#[inline]
pub fn r#rx_dp_i(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#rx_dn_i(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#rx_d_i(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#tx_dp_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#tx_dn_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(9, value); self
}
#[inline]
pub fn r#tx_d_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(10, value); self
}
#[inline]
pub fn r#tx_se0_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(11, value); self
}
#[inline]
pub fn r#tx_oe_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(12, value); self
}
#[inline]
pub fn r#pwr_sense(&mut self, value: bool) -> &mut Self {
self.reg.bit(16, value); self
}
}
pub enum r#PhyPinsDrive {}
impl register::RegSpec for r#PhyPinsDrive {
const DEFAULT: u32 = 0x0;
type Read = r#PhyPinsDriveRead;
type Write = r#PhyPinsDriveWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyPinsDriveRead { pub reg: register::RegRead<r#PhyPinsDrive> }
impl r#PhyPinsDriveRead {
#[inline]
pub fn r#dp_o(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#dn_o(self) -> bool {
self.reg.bit(1)
}
#[inline]
pub fn r#d_o(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#se0_o(self) -> bool {
self.reg.bit(3)
}
#[inline]
pub fn r#oe_o(self) -> bool {
self.reg.bit(4)
}
#[inline]
pub fn r#rx_enable_o(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#dp_pullup_en_o(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#dn_pullup_en_o(self) -> bool {
self.reg.bit(7)
}
#[inline]
pub fn r#en(self) -> bool {
self.reg.bit(16)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyPinsDriveWrite { pub reg: register::RegWrite<r#PhyPinsDrive> }
impl r#PhyPinsDriveWrite {
#[inline]
pub fn r#dp_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#dn_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
#[inline]
pub fn r#d_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#se0_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(3, value); self
}
#[inline]
pub fn r#oe_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(4, value); self
}
#[inline]
pub fn r#rx_enable_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#dp_pullup_en_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#dn_pullup_en_o(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
#[inline]
pub fn r#en(&mut self, value: bool) -> &mut Self {
self.reg.bit(16, value); self
}
}
pub enum r#PhyConfig {}
impl register::RegSpec for r#PhyConfig {
const DEFAULT: u32 = 0x4;
type Read = r#PhyConfigRead;
type Write = r#PhyConfigWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyConfigRead { pub reg: register::RegRead<r#PhyConfig> }
impl r#PhyConfigRead {
#[inline]
pub fn r#use_diff_rcvr(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#tx_use_d_se(self, index: u8) -> bool {
assert!(index < 1);
self.reg.bit(1 + index * 0)
}
#[inline]
pub fn r#eop_single_bit(self) -> bool {
self.reg.bit(2)
}
#[inline]
pub fn r#pinflip(self) -> bool {
self.reg.bit(5)
}
#[inline]
pub fn r#usb_ref_disable(self) -> bool {
self.reg.bit(6)
}
#[inline]
pub fn r#tx_osc_test_mode(self) -> bool {
self.reg.bit(7)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#PhyConfigWrite { pub reg: register::RegWrite<r#PhyConfig> }
impl r#PhyConfigWrite {
#[inline]
pub fn r#use_diff_rcvr(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#tx_use_d_se(&mut self, index: u8, value: bool) -> &mut Self {
assert!(index < 1);
self.reg.bit(1 + index * 0, value); self
}
#[inline]
pub fn r#eop_single_bit(&mut self, value: bool) -> &mut Self {
self.reg.bit(2, value); self
}
#[inline]
pub fn r#pinflip(&mut self, value: bool) -> &mut Self {
self.reg.bit(5, value); self
}
#[inline]
pub fn r#usb_ref_disable(&mut self, value: bool) -> &mut Self {
self.reg.bit(6, value); self
}
#[inline]
pub fn r#tx_osc_test_mode(&mut self, value: bool) -> &mut Self {
self.reg.bit(7, value); self
}
}
pub enum r#WakeControl {}
impl register::RegSpec for r#WakeControl {
const DEFAULT: u32 = 0x0;
type Read = r#WakeControlRead;
type Write = r#WakeControlWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WakeControlRead { pub reg: register::RegRead<r#WakeControl> }
impl r#WakeControlRead {
#[inline]
pub fn r#suspend_req(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#wake_ack(self) -> bool {
self.reg.bit(1)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WakeControlWrite { pub reg: register::RegWrite<r#WakeControl> }
impl r#WakeControlWrite {
#[inline]
pub fn r#suspend_req(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#wake_ack(&mut self, value: bool) -> &mut Self {
self.reg.bit(1, value); self
}
}
pub enum r#WakeEvents {}
impl register::RegSpec for r#WakeEvents {
const DEFAULT: u32 = 0x0;
type Read = r#WakeEventsRead;
type Write = r#WakeEventsWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WakeEventsRead { pub reg: register::RegRead<r#WakeEvents> }
impl r#WakeEventsRead {
#[inline]
pub fn r#module_active(self) -> bool {
self.reg.bit(0)
}
#[inline]
pub fn r#disconnected(self) -> bool {
self.reg.bit(8)
}
#[inline]
pub fn r#bus_reset(self) -> bool {
self.reg.bit(9)
}
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#WakeEventsWrite { pub reg: register::RegWrite<r#WakeEvents> }
impl r#WakeEventsWrite {
#[inline]
pub fn r#module_active(&mut self, value: bool) -> &mut Self {
self.reg.bit(0, value); self
}
#[inline]
pub fn r#disconnected(&mut self, value: bool) -> &mut Self {
self.reg.bit(8, value); self
}
#[inline]
pub fn r#bus_reset(&mut self, value: bool) -> &mut Self {
self.reg.bit(9, value); self
}
}
pub enum r#Buffer {}
impl register::RegSpec for r#Buffer {
const DEFAULT: u32 = 0x0;
type Read = r#BufferRead;
type Write = r#BufferWrite;
}
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#BufferRead { pub reg: register::RegRead<r#Buffer> }
#[derive(Clone, Copy, bytemuck::TransparentWrapper)] #[repr(transparent)] pub struct r#BufferWrite { pub reg: register::RegWrite<r#Buffer> }
}
