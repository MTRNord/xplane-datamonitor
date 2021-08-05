#[deny(unused_imports)]
#[deny(missing_docs)]
use std::thread;
use std::time::{Duration, SystemTime};
use xplm::{
    debug,
    flight_loop::{FlightLoop, FlightLoopCallback},
    plugin::{Plugin, PluginInfo},
    xplane_plugin,
};

use crate::location::Location;
use crate::{energy::Energy, error::Error};

mod energy;
mod error;
mod location;

struct DataMonitorPlugin {
    loophandler: LoopHandler,
    flightloop: Option<FlightLoop>,
}

#[derive(Clone)]
struct LoopHandler {
    location: Location,
    energy: Energy,
    last_run: SystemTime,
}

impl FlightLoopCallback for LoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        if self.last_run.elapsed().unwrap() >= Duration::from_secs(5) {
            let battery_on = self.energy.battery_on();
            let gpu_on = self.energy.gpu_on();
            let location = self.location.to_string();
            let energy = self.energy.to_string();
            let thread = thread::spawn(move || {
                // Do stuff with location
                if battery_on || gpu_on {
                    debug(format!("[DATAMONITOR] {}\n", location));
                    debug(format!("[DATAMONITOR] {}\n", energy));
                }
            });
            if let Err(e) = thread.join() {
                debug(format!("[DATAMONITOR][ERROR] {:?}\n", e));
            }
            self.last_run = SystemTime::now();
        }
        state.call_next_loop()
    }
}

impl DataMonitorPlugin {
    pub(crate) fn start(&mut self) {
        debug("[DATAMONITOR] starting\n");
        let mut flight_loop = FlightLoop::new(self.loophandler.clone());
        flight_loop.schedule_immediate();
        self.flightloop = Some(flight_loop);
        debug("[DATAMONITOR] started\n");
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

        let loophandler = LoopHandler {
            location: location.unwrap(),
            energy: energy.unwrap(),
            last_run: SystemTime::now(),
        };
        let plugin = DataMonitorPlugin {
            loophandler,
            flightloop: None,
        };
        Ok(plugin)
    }
    fn enable(&mut self) -> Result<(), Self::Error> {
        self.start();
        Ok(())
    }
    fn disable(&mut self) {
        debug("[DATAMONITOR][INFO] Stopping threads\n");
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
