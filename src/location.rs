use std::{fmt::Display, rc::Rc};

use xplm::data::{borrowed::DataRef, DataRead, ReadOnly};

use crate::error::Error;

pub(crate) struct LocationRefs {
    pub(crate) latitude: DataRef<f64, ReadOnly>,
    pub(crate) longitude: DataRef<f64, ReadOnly>,
}

#[derive(Clone)]
pub(crate) struct Location {
    inner: Rc<LocationRefs>,
}

impl Location {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: Rc::new(LocationRefs {
                latitude: DataRef::find("sim/flightmodel/position/latitude")?,
                longitude: DataRef::find("sim/flightmodel/position/longitude")?,
            }),
        })
    }

    pub(crate) fn lat(&self) -> f64 {
        self.inner.latitude.get()
    }
    pub(crate) fn lon(&self) -> f64 {
        self.inner.longitude.get()
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Latitude: {}, Longitude: {}",
            self.inner.latitude.get(),
            self.inner.longitude.get()
        )
    }
}
