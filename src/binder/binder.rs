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

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use errors::*;
use nix::fcntl::*;
use nix::fcntl::OFlag;
use nix::sys::mman::*;
use nix::sys::stat::Mode;
use nix::unistd::close;
use std::mem::size_of;
use std::os::unix::io::RawFd;
use std::slice::from_raw_parts;
use types::*;
use utils::any_as_u8_slice;

const DEVICE: &str = "/dev/binder";
#[cfg(feature = "binder_version_7")]
const BINDER_PROTOCOL_VERSION: i32 = 7;
#[cfg(feature = "binder_version_8")]
const BINDER_PROTOCOL_VERSION: i32 = 8;
const BINDER_IOC_MAGIC: u8 = b'b';
const READ_SIZE: usize = 32 * 4;
const MAX_THREADS: usize = 15;
const MAP_SIZE: usize = ((1 * 1024 * 1024) - (4096 * 2));

ioctl!(readwrite binder_write_read with BINDER_IOC_MAGIC, 1; BinderWriteRead);
ioctl!(write_ptr _binder_set_idle_timeout with BINDER_IOC_MAGIC, 3; i64);
ioctl!(write_ptr binder_set_max_threads with BINDER_IOC_MAGIC, 5; usize);
ioctl!(write_ptr _binder_set_idle_priotity with BINDER_IOC_MAGIC, 6; i32);
ioctl!(write_int _binder_set_context_mgr with BINDER_IOC_MAGIC, 7);
ioctl!(write_int binder_thread_exit with BINDER_IOC_MAGIC, 8);
ioctl!(readwrite binder_version with BINDER_IOC_MAGIC, 9; BinderVersion);

pub struct Binder {
    fd: RawFd,
    mapped: Vec<u8>,
}

#[repr(packed)]
pub struct WriteBuffer {
    command: BinderDriverCommandProtocol,
    transaction: BinderTransactionData,
}

#[derive(Debug)]
pub enum Reply {
    Data(Vec<u8>),
    StatusCode(u32),
}

#[derive(Debug)]
pub enum CallResult {
    Noop,
    Reply(Reply),
}

impl<'a> Binder {
    pub fn new() -> Result<Binder> {
        let mut flags = OFlag::empty();
        flags.set(O_RDWR, true);
        flags.set(O_CLOEXEC, true);

        let fd = open(DEVICE, flags, Mode::empty()).chain_err(|| {
            format!("Failed to open {}", DEVICE)
        })?;

        if !unsafe {
            let mut version_data = BinderVersion::default();
            binder_version(fd, &mut version_data)
                .chain_err(|| "Failed to get version")
                .map(|_| version_data.protocol_version as i32)
                .map(|d| d == BINDER_PROTOCOL_VERSION)?
        }
        {
            return Err("Binder protocol version mismatch".into());
        }
        info!("Binder version check passed");

        let mut binder = Binder {
            fd,
            mapped: vec![0; MAP_SIZE],
        };

        let mut prot_flags = ProtFlags::empty();
        prot_flags.set(PROT_READ, true);
        let mut flags = MapFlags::empty();
        flags.set(MAP_PRIVATE, true);
        let mapped = (&mut binder.mapped).as_mut_ptr() as *mut ::nix::libc::c_void;
        unsafe {
            mmap(mapped, MAP_SIZE, prot_flags, flags, fd, 0).chain_err(
                || "Failed to mmap",
            )?;
        }
        info!("Mapped {} bytes", MAP_SIZE);

        info!("Setting max threads to {}", MAX_THREADS);
        unsafe {
            binder_set_max_threads(fd, &MAX_THREADS).chain_err(
                || "Failed to set max threads",
            )?;
        };

        info!("Entering looper");
        let mut data = vec![];
        data.write_u32::<LittleEndian>(BinderDriverCommandProtocol::BC_ENTER_LOOPER as u32)
            .unwrap();
        let mut d = BinderWriteRead {
            write_size: data.len() as BinderSize,
            write_buffer: (&mut data).as_mut_ptr() as BinderPtr,
            ..Default::default()
        };
        unsafe {
            binder_write_read(fd, &mut d).chain_err(
                || "Failed to enter looper",
            )?;
        };

        Ok(binder)
    }

    pub fn call(&self, msg: &[u8], target: u32, code: u32, flags: u32) -> Result<CallResult> {
        let write_buffer = WriteBuffer {
            command: BinderDriverCommandProtocol::BC_TRANSACTION,
            transaction: BinderTransactionData {
                target,
                cookie: 0,
                code,
                flags: flags,
                sender_pid: 0,
                sender_euid: 0,
                data_size: msg.len() as BinderSize,
                offsets_size: 0,
                data: msg.as_ptr() as BinderPtr,
            },
        };

        let mut read_buffer: [u8; READ_SIZE] = [0; READ_SIZE];
        let mut bwr = BinderWriteRead {
            write_size: size_of::<WriteBuffer>() as BinderSize,
            write_consumed: 0,
            write_buffer: (&write_buffer as *const WriteBuffer) as BinderPtr,
            read_size: size_of::<[u8; READ_SIZE]>() as BinderSize,
            read_consumed: 0,
            read_buffer: (&mut read_buffer as *mut [u8; READ_SIZE]) as BinderPtr,
        };

        debug!("Call transaction data:");
        hex!(msg);

        debug!("Writing:");
        hex!(any_as_u8_slice(&bwr));

        unsafe {
            binder_write_read(self.fd, &mut bwr).chain_err(|| "Failed to write/read")?;
        };
        let mut d = &read_buffer[..(bwr.read_consumed as usize)];
        debug!("Reply data:");
        hex!(&d);

        loop {
            let c: BinderDriverReturnProtocol = d.read_u32::<LittleEndian>()
                .chain_err(|| "Invalid read reply")?.into();
            info!("BinderDriverReturnProtocol is {:?}", c);

            match c {
                BinderDriverReturnProtocol::BR_NOOP => (),
                BinderDriverReturnProtocol::BR_ERROR => return Err("Binder error".into()),
                BinderDriverReturnProtocol::BR_TRANSACTION_COMPLETE => (),
                BinderDriverReturnProtocol::BR_SPAWN_LOOPER => (),
                BinderDriverReturnProtocol::BR_REPLY => {
                   if d.len() < size_of::<BinderTransactionData>() {
                       return Err(format!("Reply data to short: {} vs {}", d.len(), size_of::<BinderTransactionData>()).into());
                   }

                   let td: BinderTransactionData = unsafe { ::std::ptr::read(d.as_ptr() as *const _) };
                   // d = &d[size_of::<binder_transaction_data>()..];

                   debug!("Target: {:?} Cookie: {:?} Code: {}", td.target, td.cookie, td.code);
                   debug!("Flags: {:x}", td.flags);
                   debug!("Sender pid: {} euid: {}", td.sender_pid, td.sender_euid);
                   debug!("Data size: {} Offsets size: {}", td.data_size, td.offsets_size);

                   if (td.flags & TransactionFlags::STATUS_CODE as u32) != 0 {
                       let code = unsafe { *(td.data as *const u32) };
                       debug!("Status code: {:x}", code);
                       return Ok(CallResult::Reply(Reply::StatusCode(code)));
                   }

                   let r = if td.data_size > 0 {
                       unsafe {
                           let p = td.data as *const u8;
                           from_raw_parts(p, td.data_size as usize).to_vec()
                       }
                   } else {
                       vec![]
                   };
                   if !r.is_empty() {
                       debug!("Data:");
                       hex!(&r);
                   }
                   return Ok(CallResult::Reply(Reply::Data(r)));
                },
                BinderDriverReturnProtocol::BR_FAILED_REPLY => return Err("Transaction failed".into()),
                _ => unimplemented!(),

            }

            if d.is_empty() {
                return Ok(CallResult::Noop);
            }
        }
    }

    pub fn serve(&self) -> Result<()> {
        info!("Entering looper");
        let mut data = vec![];
        data.write_u32::<LittleEndian>(BinderDriverCommandProtocol::BC_ENTER_LOOPER as u32)
            .unwrap();
        let mut d = BinderWriteRead {
            write_size: data.len() as BinderSize,
            write_buffer: (&mut data).as_mut_ptr() as BinderPtr,
            ..Default::default()
        };
        unsafe {
            binder_write_read(self.fd, &mut d).chain_err(
                || "Failed to enter looper",
            )?;
        };

        loop {
            let mut read_buffer: [u8; READ_SIZE] = [0; READ_SIZE];
            let bwr = BinderWriteRead {
                write_buffer: 0,
                read_size: size_of::<[u8; READ_SIZE]>() as BinderSize,
                read_buffer: (&mut read_buffer as *mut [u8; READ_SIZE]) as BinderPtr,
                ..Default::default()
            };
            unsafe {
                binder_write_read(self.fd, &mut d).chain_err(
                    || "Failed to enter looper",
                )?;
            };

            let mut d = &read_buffer[..(bwr.read_consumed as usize)];

            while ! d.is_empty() {
                let c: BinderDriverReturnProtocol = d.read_u32::<LittleEndian>()
                    .chain_err(|| "Invalid read reply")?.into();
                info!("BinderDriverReturnProtocol is {:?}", c);

                match c {
                    BinderDriverReturnProtocol::BR_NOOP => (),
                    BinderDriverReturnProtocol::BR_ERROR => return Err("Binder error".into()),
                    BinderDriverReturnProtocol::BR_TRANSACTION_COMPLETE => (),
                    BinderDriverReturnProtocol::BR_SPAWN_LOOPER => (),
                    BinderDriverReturnProtocol::BR_REPLY => (),
                    BinderDriverReturnProtocol::BR_FAILED_REPLY => return Err("Transaction failed".into()),
                    _ => unimplemented!(),

                }
            }
        }
    }

}

impl Drop for Binder {
    fn drop(&mut self) {
        unsafe {
            binder_thread_exit(self.fd, 0).expect("Failed to exit binder");
        }
        close(self.fd).unwrap_or_else(|_| {
            error!("Failed to close");
        });
        info!("Dropped binder with fd {}", self.fd);
    }
}
