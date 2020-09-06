#![feature(vec_remove_item)]
#![feature(iter_partition_in_place)]

extern crate fast_floats;
extern crate fixedbitset;

pub use self::common::exclusion::{InterventionExclusions, Seasons};
pub use self::common::intervention::{Intervention, Workload};
pub use self::common::risks::Risks;
pub use self::common::types::{Day, Period, IID, PID, PRID, RID, SID};
pub use self::common::{
    Maintenance, MaintenanceMapping, MaintenanceOptimization, Planning, Resource,
};

pub mod builder {
    pub use crate::common::intervention::InterventionBuilder;
    pub use crate::common::MaintenanceBuilder;
}

mod common;
pub mod search;
mod utils;
