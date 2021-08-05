use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use xplm::data::{borrowed::DataRef, DataRead, ReadOnly};

use crate::error::Error;

pub(crate) struct LocationRefs {
    pub(crate) latitude: DataRef<f64, ReadOnly>,
    pub(crate) longitude: DataRef<f64, ReadOnly>,
}

unsafe impl Send for LocationRefs {}
unsafe impl Sync for LocationRefs {}

#[derive(Clone)]
pub(crate) struct Location {
    inner: Arc<RwLock<LocationRefs>>,
}

impl Location {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: Arc::new(RwLock::new(LocationRefs {
                latitude: DataRef::find("sim/flightmodel/position/latitude")?,
                longitude: DataRef::find("sim/flightmodel/position/longitude")?,
            })),
        })
    }

    pub(crate) fn lat(&self) -> Result<f64, Error> {
        let lock = self.inner.read().or(Err(Error::UnableToGetLock))?;
        Ok(lock.latitude.get())
    }
    pub(crate) fn lon(&self) -> Result<f64, Error> {
        let lock = self.inner.read().or(Err(Error::UnableToGetLock))?;
        Ok(lock.longitude.get())
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lock = self.inner.read();
        if let Ok(lock) = lock {
            write!(
                f,
                "Latitude: {}, Longitude: {}",
                lock.latitude.get(),
                lock.longitude.get()
            )
        } else {
            Ok(())
        }
    }
}
