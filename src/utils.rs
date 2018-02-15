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

use std::mem::size_of;
use std::slice::from_raw_parts;

macro_rules! hex {
    ($x:expr) => {
        {
            for l in ::hexdump::hexdump_iter($x) {
                debug!("{}", l);
            }
        }
    };
}

#[allow(dead_code)]
pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { from_raw_parts((p as *const T) as *const u8, size_of::<T>()) }
}
