#[deny(unused_imports)]
#[deny(missing_docs)]
use influxdb_client::{Client, Point, Precision, TimestampOptions};
use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::runtime::{self, Runtime};
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
    last_run: Instant,
    start_time: SystemTime,
    leg_started: bool,
    influx: Arc<influxdb_client::Client>,
    rt: Arc<Runtime>,
}

impl FlightLoopCallback for LoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        let battery_on = self.energy.battery_on();
        let gpu_on = self.energy.gpu_on();
        let apu_on = self.energy.apu_on();
        if (battery_on || gpu_on || apu_on) && !self.leg_started {
            // TODO reset on FP change
            self.leg_started = true;
        }
        if self.last_run.elapsed() >= Duration::from_secs(1) && self.leg_started {
            let latitude = self.location.lat();
            let longitude = self.location.lon();
            let altitude = self.location.alt();
            let timestamp = self
                .start_time
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64;

            let client = self.influx.clone();
            self.rt.spawn(async move {
                let location_point = Point::new("location")
                    .tag("start_time", timestamp)
                    .field("latitude", latitude)
                    .field("longitude", longitude)
                    .field("altitude_feet", altitude);
                let status_point = Point::new("status")
                    .tag("start_time", timestamp)
                    .field("battery_on", battery_on)
                    .field("gpu_on", gpu_on)
                    .field("apu_on", apu_on);
                let points = vec![&location_point, &status_point];
                if let Err(e) = client.insert_points(points, TimestampOptions::None).await {
                    debug(format!("[DATAMONITOR][ERROR] Sending Points: {:#?}\n", e));
                };
            });
            self.last_run = Instant::now();
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

        let influx_client = Client::new("http://10.0.0.1:8086", 
        "wuL5_5sg_zlaQdkDWhmiFZ9r-Fx1rWgNR407czXOeQmYU1PHlp0nwpmjjW270PzEgEctx0AqD_K7K-h9Ein6Pg==")
        .with_org("Nordgedanken")
    .with_bucket("flightdata")
    .with_precision(Precision::MS);

        let rt = runtime::Runtime::new();
        if let Err(ref e) = rt {
            debug(format!("[DATAMONITOR][ERROR] Creating runtime: {:#?}\n", e));
        }

        let loophandler = LoopHandler {
            location: location.unwrap(),
            energy: energy.unwrap(),
            last_run: Instant::now(),
            influx: Arc::new(influx_client),
            start_time: SystemTime::now(),
            leg_started: false,
            rt: Arc::new(rt.unwrap()),
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
