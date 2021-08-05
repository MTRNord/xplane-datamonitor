use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use xplm::data::{borrowed::DataRef, ArrayRead, DataRead, ReadWrite};

use crate::error::Error;

pub(crate) struct EnergyRefs {
    pub(crate) gpu_on: DataRef<bool, ReadWrite>,
    pub(crate) battery: DataRef<[i32], ReadWrite>,
}

unsafe impl Send for EnergyRefs {}
unsafe impl Sync for EnergyRefs {}

#[derive(Clone)]
pub(crate) struct Energy {
    inner: Arc<RwLock<EnergyRefs>>,
}

impl Energy {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: Arc::new(RwLock::new(EnergyRefs {
                gpu_on: DataRef::find("sim/cockpit/electrical/gpu_on")?.writeable()?,
                battery: DataRef::find("sim/cockpit2/electrical/battery_on")?.writeable()?,
            })),
        })
    }

    pub(crate) fn gpu_on(&self) -> Result<bool, Error> {
        let lock = self.inner.read().or(Err(Error::UnableToGetLock))?;
        Ok(lock.gpu_on.get())
    }

    pub(crate) fn battery_on(&self) -> Result<bool, Error> {
        let lock = self.inner.read().or(Err(Error::UnableToGetLock))?;
        Ok(lock.battery.as_vec().contains(&1))
    }
}

impl Display for Energy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lock = self.inner.read();
        if let Ok(lock) = lock {
            write!(
                f,
                "GPU enabled: {}, Battery enabled: {}",
                lock.gpu_on.get(),
                lock.battery.as_vec().contains(&1),
            )
        } else {
            Ok(())
        }
    }
}
