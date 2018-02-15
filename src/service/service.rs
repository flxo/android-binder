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

use errors::*;
use binder::binder::{CallResult, Reply, Binder};
use super::Parcel;

pub struct Service {
    handle: u32,
    binder: Binder,
}

impl Service {
    pub fn new(h: u32, b: Binder) -> Service {
        Service {
            handle: h,
            binder: b,
        }
    }

    pub fn call(&self, code: u32, parcel: &Parcel, flags: u32) -> Result<Parcel> {
        let r = self.binder.call(&parcel, self.handle, code, flags)?;
        if let CallResult::Reply(r) = r {
            match r {
                Reply::Data(r) => {
                    return Ok(Parcel::from_buf(&r));
                }
                _ => unimplemented!(),
            }
        }
        unimplemented!()
    }
}
