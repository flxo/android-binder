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
use std::mem::size_of;
use std::ops::Deref;
use types::{BinderType, BinderPtr, FlatBinderObject, FlatBinderFlags};
use utils::any_as_u8_slice;

const STRICT_MODE_PENALTY_GATHER: i32 = 0x40 << 16;

pub enum Object {
    Handle(u32),
    Binder(*mut ()),
}

#[derive(Default, Debug)]
pub struct Parcel {
    data: Vec<u8>,
}

impl Deref for Parcel {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Parcel {
    pub fn from_buf(d: &[u8]) -> Parcel {
        Parcel { data: d.to_vec() }
    }

    pub fn put_interface_token(&mut self, interface: &str) -> Result<()> {
        self.put_i32(STRICT_MODE_PENALTY_GATHER)?; // strict mode
        self.put_str16(interface) // interface token
    }

    pub fn put_u8(&mut self, n: u8) -> Result<()> {
        self.data.push(n);
        Ok(())
    }

    pub fn put_i16(&mut self, n: i16) -> Result<()> {
        self.data.write_i16::<LittleEndian>(n).chain_err(
            || "Failed to put i16",
        )
    }

    pub fn put_u16(&mut self, n: u16) -> Result<()> {
        self.data.write_u16::<LittleEndian>(n).chain_err(
            || "Failed to put u16",
        )
    }

    pub fn put_i32(&mut self, n: i32) -> Result<()> {
        self.data.write_i32::<LittleEndian>(n).chain_err(
            || "Failed to put i32",
        )
    }

    pub fn put_u32(&mut self, n: u32) -> Result<()> {
        self.data.write_u32::<LittleEndian>(n).chain_err(
            || "Failed to put u32",
        )
    }

    pub fn put_str16(&mut self, s: &str) -> Result<()> {
        self.data.reserve(size_of::<i32>() + s.len() * 2 + 2);
        self.put_i32((s.len()) as i32)?;
        for c in s.encode_utf16() {
            self.put_u16(c)?;
        }
        self.put_u16(0)?; // zero termination
        // padding
        if (self.data.len() % 4) != 0 {
            let l = self.data.len();
            self.data.resize(l + 4 - (l % 4), 0);
        }
        Ok(())
    }

    // TODO: Understand how to transmit a binder.
    pub fn put_binder(&mut self, binder: BinderPtr, cookie: BinderPtr) -> Result<()> {
        let o = FlatBinderObject {
            type_: BinderType::Binder as u32,
            flags: 0x7F | FlatBinderFlags::AcceptFds as u32,
            handle_binder: binder as BinderPtr,
            cookie: cookie as BinderPtr,
        };
        self.data.extend(any_as_u8_slice(&o));
        if (self.data.len() % 4) != 0 {
            let l = self.data.len();
            self.data.resize(l + 4 - (l % 4), 0);
        }
        Ok(())
    }

    // pub fn put_object(&mut self, type_: BinderType, object: Object, _cookie: ()) -> Result<()> {
    //     match object {
    //         Object::Handle(h) => {
    //             let o = FlatBinderObject {
    //                 type_: type_ as u32,
    //                 flags: 0x7F | FlatBinderFlags::AcceptFds as u32,
    //                 handle: h,
    //                 cookie: self.as_ptr() as BinderPtr,
    //             };
    //             self.data.extend(any_as_u8_slice(&o));
    //             if (self.data.len() % 4) != 0 {
    //                 let l = self.data.len();
    //                 self.data.resize(l + 4 - (l % 4), 0);
    //             }
    //         },
    //         Object::Binder(p) => {
    //             let o = FlatBinderObject {
    //                 type_: type_ as u32,
    //                 flags: 0x7F | FlatBinderFlags::AcceptFds as u32,
    //                 handle: p as BinderPtr,
    //                 cookie: self.as_ptr() as BinderPtr,
    //             };
    //         },
    //         _ => unimplemented!(),
    //     }
    //     Ok(())
    // }

    pub fn get_i32(&mut self) -> Result<i32> {
        let r = self.data.as_slice().read_i32::<LittleEndian>().chain_err(
            || "Data exhausted",
        )?;
        self.data.drain(..size_of::<i32>());
        Ok(r)
    }

    pub fn get_str16(&mut self) -> Result<String> {
        let l = self.get_i32()? as usize;
        debug!("length: {}", l);
        let d = self.data.drain(..((l * 2) + 2)).collect::<Vec<u8>>();
        let r: &[u16] = {
            unsafe { ::std::slice::from_raw_parts(d.as_slice().as_ptr() as *const u16, l * 2) }
        };
        Ok(String::from_utf16(r).chain_err(|| "Invlid string")?)
    }


    pub fn get_obj(&mut self) -> Result<Object> {
        let d = self.data.drain(..size_of::<FlatBinderObject>()).collect::<Vec<u8>>();
        let o: FlatBinderObject = unsafe { ::std::ptr::read((&d).as_ptr() as *const _) };
        let t: BinderType = o.type_.into();
        match t {
            BinderType::Handle => {
                let h = o.handle_binder as u32;
                return Ok(Object::Handle(h));
            },
            _ => unimplemented!(),
        }
    }
}
