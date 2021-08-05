use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::JoinHandle,
};
#[deny(unused_imports)]
#[deny(missing_docs)]
use std::{thread, time};
use tokio::runtime::Runtime;
use xplm::{
    debug,
    plugin::{Plugin, PluginInfo},
    xplane_plugin,
};

use crate::location::Location;
use crate::{energy::Energy, error::Error};

mod energy;
mod error;
mod location;

#[derive(Clone)]
struct DataMonitorPlugin {
    location: Location,
    energy: Energy,
    thread: Arc<RwLock<Option<JoinHandle<()>>>>,
    stopped: Arc<AtomicBool>,
}

impl DataMonitorPlugin {
    pub(crate) fn start(&self) {
        debug("[DATAMONITOR] starting\n");
        let self_clone = self.clone();
        let thread = thread::spawn(move || {
            let delay = time::Duration::from_secs(5);
            {
                debug(format!("[DATAMONITOR] {}\n", self_clone.location));
            };
            let rt = Runtime::new().unwrap();
            loop {
                // Do stuff
                if self_clone.stopped.load(Ordering::Relaxed) {
                    break;
                }

                let current_location = self_clone.location.clone();
                let current_energy = self_clone.energy.clone();
                rt.spawn(async move {
                    // Do stuff with location
                    if current_energy.battery_on().unwrap() || current_energy.gpu_on().unwrap() {
                        debug(format!("[DATAMONITOR] {}\n", current_location));
                        debug(format!("[DATAMONITOR] {}\n", current_energy));
                    }
                });
                thread::sleep(delay);
            }
        });
        let mut lock = self.thread.write().unwrap();
        *lock = Some(thread);
    }
}

impl Plugin for DataMonitorPlugin {
    type Error = Error;

    fn start() -> Result<Self, Self::Error> {
        let location = Location::new();
        if let Err(ref e) = location {
            debug(format!("[DATAMONITOR][ERROR] Location init: {:?}\n", e));
        }
        let energy = Energy::new();
        if let Err(ref e) = energy {
            debug(format!("[DATAMONITOR][ERROR] Energy init: {:?}\n", e));
        }
        let plugin = DataMonitorPlugin {
            location: location.unwrap(),
            energy: energy.unwrap(),
            thread: Arc::new(RwLock::new(None)),
            stopped: Arc::new(AtomicBool::new(false)),
        };
        Ok(plugin)
    }
    fn enable(&mut self) -> Result<(), Self::Error> {
        self.start();
        Ok(())
    }
    fn disable(&mut self) {
        debug("[DATAMONITOR][INFO] Stopping threads\n");
        self.stopped.swap(true, Ordering::Release);
        let mut lock = self.thread.write().unwrap();
        let thread = std::mem::replace(&mut *lock, None);
        if let Some(thread) = thread {
            thread.join().unwrap();
            debug("[DATAMONITOR][INFO] Stopped threads\n");
        }
    }

    fn info(&self) -> xplm::plugin::PluginInfo {
        PluginInfo {
            name: String::from("Datamonitor"),
            signature: String::from("dev.nordgedanken.datamonitor"),
            description: String::from(
                "Gets certain datarefs to display in a grafana for review of flights.",
            ),
        }
    }
}

xplane_plugin!(DataMonitorPlugin);
