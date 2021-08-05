use std::{fmt::Display, rc::Rc};

use xplm::data::{borrowed::DataRef, ArrayRead, DataRead, ReadWrite};

use crate::error::Error;

pub(crate) struct EnergyRefs {
    pub(crate) gpu_on: DataRef<bool, ReadWrite>,
    pub(crate) battery: DataRef<[i32], ReadWrite>,
}

#[derive(Clone)]
pub(crate) struct Energy {
    inner: Rc<EnergyRefs>,
}

impl Energy {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: Rc::new(EnergyRefs {
                gpu_on: DataRef::find("sim/cockpit/electrical/gpu_on")?.writeable()?,
                battery: DataRef::find("sim/cockpit2/electrical/battery_on")?.writeable()?,
            }),
        })
    }

    pub(crate) fn gpu_on(&self) -> bool {
        self.inner.gpu_on.get()
    }

    pub(crate) fn battery_on(&self) -> bool {
        self.inner.battery.as_vec().contains(&1)
    }
}

impl Display for Energy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GPU enabled: {}, Battery enabled: {}",
            self.inner.gpu_on.get(),
            self.inner.battery.as_vec().contains(&1),
        )
    }
}
