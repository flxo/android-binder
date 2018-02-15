// Copyright (C) 2017 Felix Obenhuber
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::mem::transmute;

#[cfg(feature = "binder_version_7")]
pub type BinderSize = u32;
#[cfg(feature = "binder_version_8")]
pub type BinderSize = u64;
#[cfg(feature = "binder_version_7")]
pub type BinderPtr = u32;
#[cfg(feature = "binder_version_8")]
pub type BinderPtr = u64;
pub type BinderPid = i32;
pub type BinderUid = u32;

#[repr(u32)]
pub enum FlatBinderFlags {
  PriorityMask = 0xff,
  AcceptFds = 0x100,
}

macro_rules! pack_chars {
    ($c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
        (((($c1 as u32) << 24)) | ((($c2 as u32) << 16)) | ((($c3 as u32) << 8)) | ($c4 as u32))
    };
}

const BINDER_TYPE_LARGE: u8 = 0x85;

#[repr(u32)]
pub enum BinderType {
    Binder = pack_chars!(b's', b'b', b'*', BINDER_TYPE_LARGE),
    WeakBinder = pack_chars!(b'w', b'b', b'*', BINDER_TYPE_LARGE),
    Handle = pack_chars!(b's', b'h', b'*', BINDER_TYPE_LARGE),
    WeakHandle = pack_chars!(b'w', b'h', b'*', BINDER_TYPE_LARGE),
    Fd = pack_chars!(b'f', b'd', b'*', BINDER_TYPE_LARGE),
    Fda = pack_chars!(b'f', b'd', b'a', BINDER_TYPE_LARGE),
    Ptr = pack_chars!(b'p', b't', b'*', BINDER_TYPE_LARGE),
}

impl From<u32> for BinderType {
    fn from(v: u32) -> BinderType {
        unsafe { transmute(v) }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct BinderVersion {
    pub protocol_version: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct BinderWriteRead {
    pub write_size: BinderSize,
    pub write_consumed: BinderSize,
    pub write_buffer: BinderPtr,
    pub read_size: BinderSize,
    pub read_consumed: BinderSize,
    pub read_buffer: BinderPtr,
}

impl Default for BinderWriteRead {
    fn default() -> Self {
        BinderWriteRead {
            write_size: 0,
            write_consumed: 0,
            write_buffer: 0,
            read_size: 0,
            read_consumed: 0,
            read_buffer: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BinderTransactionData {
    pub target: u32,
    pub cookie: BinderPtr,
    pub code: u32,
    pub flags: u32,
    pub sender_pid: BinderPid,
    pub sender_euid: BinderUid,
    pub data_size: BinderSize,
    pub offsets_size: BinderSize,
    pub data: BinderPtr,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TransactionFlags {
    ONE_WAY = 1,
    ROOT_OBJECT = 4,
    STATUS_CODE = 8,
    ACCEPT_FDS = 16,
}

// TODO
#[repr(u32)]
#[derive(Debug)]
pub enum BinderDriverCommandProtocol {
    BC_TRANSACTION = 1076388608,
    BC_REPLY = 1076388609,
    BC_ACQUIRE_RESULT = 1074029314,
    BC_FREE_BUFFER = 1074029315,
    BC_INCREFS = 1074029316,
    BC_ACQUIRE = 1074029317,
    BC_RELEASE = 1074029318,
    BC_DECREFS = 1074029319,
    BC_INCREFS_DONE = 1074291464,
    BC_ACQUIRE_DONE = 1074291465,
    BC_ATTEMPT_ACQUIRE = 1074291466,
    BC_REGISTER_LOOPER = 25355,
    BC_ENTER_LOOPER = 25356,
    BC_EXIT_LOOPER = 25357,
    BC_REQUEST_DEATH_NOTIFICATION = 1074291470,
    BC_CLEAR_DEATH_NOTIFICATION = 1074291471,
    BC_DEAD_BINDER_DONE = 1074029328,
}

// TODO
#[repr(u32)]
#[derive(Debug)]
pub enum BinderDriverReturnProtocol {
    BR_ERROR = 2147774976,
    BR_OK = 29185,
    BR_TRANSACTION = 2150134274,
    BR_REPLY = 2150134275,
    BR_ACQUIRE_RESULT = 2147774980,
    BR_DEAD_REPLY = 29189,
    BR_TRANSACTION_COMPLETE = 29190,
    BR_INCREFS = 2148037127,
    BR_ACQUIRE = 2148037128,
    BR_RELEASE = 2148037129,
    BR_DECREFS = 2148037130,
    BR_ATTEMPT_ACQUIRE = 2148299275,
    BR_NOOP = 29196,
    BR_SPAWN_LOOPER = 29197,
    BR_FINISHED = 29198,
    BR_DEAD_BINDER = 2147774991,
    BR_CLEAR_DEATH_NOTIFICATION_DONE = 2147774992,
    BR_FAILED_REPLY = 29201,
}

impl From<u32> for BinderDriverReturnProtocol {
    fn from(v: u32) -> Self {
        unsafe { ::std::mem::transmute(v) }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Transaction {
    FirstCall = 1,
    LastCall = 16777215,
    Ping = 1599098439,
    Dump = 1598311760,
    Interface = 1598968902,
    Sysprops = 1599295570,
}

pub const FLAG_ONEWAY: Transaction = Transaction::FirstCall;

#[repr(C)]
#[derive(Debug)]
pub struct FlatBinderObject {
    pub type_: u32,
    pub flags: u32,
    pub handle_binder: BinderPtr, // TODO
    pub cookie: BinderPtr,
}
