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
//

extern crate android_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate android_binder;

use errors::*;
use error_chain::ChainedError;
use android_binder::service::ServiceManager;

mod errors {
    error_chain! {
        foreign_links {
            AndroidBinder(::android_binder::errors::Error);
        }
    }
}

fn run() -> Result<i32> {
    //let service_manager = ServiceManager::new()?;
    // info!("Getting permission service");
    // let permission = service_manager.get_service("permission").chain_err(
    //     || "Failed to get permission service",
    // )?;
    // debug!("Calling code 1 on permission controller");
    // let mut p = Parcel::default();
    // p.put_interface_token("android.os.IPermissionController").chain_err(|| "Failed to prep")?;
    // let r = permission.call(1, &p, 0).chain_err(|| "Failed to call")?;
    // debug!("Permission control result: {:?}", r);

    info!("Available services:");
    let service_manager = ServiceManager::new()?;
    let services = service_manager.list_services()?;
    for s in services {
        debug!("  {}", s);
    }

    Ok(0)
}

fn main() {
    android_logger::init_once(log::LogLevel::Trace);
    if let Err(ref e) = run() {
        error!("{}", e.display_chain());
        std::process::exit(1);
    }
}
