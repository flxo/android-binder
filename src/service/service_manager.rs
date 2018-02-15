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

use binder::binder::{CallResult, Reply, Binder};
use errors::*;
use types::*;
use super::parcel::{Parcel, Object};
use super::Service;

const BINDER_SERVICE_MANAGER: u32 = 0;

const SVC_MGR_GET_SERVICE: u32 = 1;
const _SVC_MGR_CHECK_SERVICE: u32 = 2;
const SVC_MGR_ADD_SERVICE: u32 = 3;
const SVC_MGR_LIST_SERVICES: u32 = 4;

const INTERFACE_SERVICE_MANAGER: &str = "android.os.IServiceManager";

pub struct ServiceManager {
    binder: Binder,
}

impl ServiceManager {
    pub fn new() -> Result<ServiceManager> {
        let s: ServiceManager = Binder::new()?.into();
        s.ping()?;
        Ok(s)
    }

    fn ping(&self) -> Result<()> {
        info!("Pingging service manager");
        let d = Parcel::default();
        self.binder.call(
            &d,
            BINDER_SERVICE_MANAGER,
            Transaction::Ping as u32,
            0x10,
        )?;
        Ok(())
    }

    pub fn get_service(self, name: &str) -> Result<Service> {
        let mut p = Parcel::default();
        p.put_interface_token(INTERFACE_SERVICE_MANAGER)?;
        p.put_str16(name)?;
        let r = self.binder.call(
            &p,
            BINDER_SERVICE_MANAGER,
            SVC_MGR_GET_SERVICE,
            0,
        )?;

        if let CallResult::Reply(r) = r {
            match r {
                Reply::Data(d) => {
                    let mut p = Parcel::from_buf(&d);
                    info!("Received parcel with {} bytes", p.len());
                    match p.get_obj()? {
                        Object::Handle(h) => {
                            debug!("Received handle {}", h);
                            return Ok(Service::new(h, self.binder));
                        },
                        _ => unimplemented!(),
                    }
                },
                _ => unimplemented!(),
            }
        }
        Err("Invalid reply for get service call".into())
    }

    pub fn add_service(self, name: &str, allow_isolated: bool) -> Result<()> {
        let mut p = Parcel::default();
        p.put_interface_token(INTERFACE_SERVICE_MANAGER)?;
        p.put_str16(name)?;
        p.put_binder(0xABABABAB as BinderPtr, 0xCACACACA as BinderPtr)?; 
        p.put_i32(if allow_isolated { 1 } else { 0 })?;
        match self.binder.call(&p, BINDER_SERVICE_MANAGER, SVC_MGR_ADD_SERVICE, 0) {
            Ok(CallResult::Reply(Reply::StatusCode(c))) => {
                warn!("Received status code {}", c);
                return Err("Failed to add service".into());
            },
            Ok(CallResult::Reply(Reply::Data(d))) => {
                info!("Received data:");
                hex!(&d);
            },
            _ => unimplemented!(),
        }
        Ok(())
    }

    pub fn list_services(&self) -> Result<Vec<String>> {
        let mut result = Vec::new();

        for n in 0..::std::u32::MAX {
            info!("Getting service number {}", n);
            let mut data = Parcel::default();
            data.put_interface_token(INTERFACE_SERVICE_MANAGER)?;
            data.put_u32(n)?;
            if let Ok(r) = self.binder.call(&data, BINDER_SERVICE_MANAGER, SVC_MGR_LIST_SERVICES, 0) {
                if let CallResult::Reply(r) = r {
                    match r {
                        Reply::Data(d) => {
                            let mut p = Parcel::from_buf(&d);
                            let svc =  p.get_str16()?;
                            debug!("service: {}", svc);
                            result.push(svc);
                        }
                        Reply::StatusCode(_) => break,
                    }
                }
            } else {
                info!("Finished after {} services", result.len());
                break;
            }
        }
        Ok(result)
    }
}

impl Into<ServiceManager> for Binder {
    fn into(self) -> ServiceManager {
        ServiceManager { binder: self }
    }
}
