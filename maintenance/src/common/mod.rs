pub mod exclusion;
pub mod intervention;
pub mod risks;
pub mod types;

use std::collections::HashMap;
use std::time::Duration;

use crate::common::exclusion::*;
use crate::common::intervention::Intervention;
use crate::common::types::*;

//pub struct UnsafeMutFixArray<T> {
//array: Box<[T]>,
//}

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct UnsafeFixArray<T> {
    array: Box<[T]>,
}

impl<T> std::ops::Index<usize> for UnsafeFixArray<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        unsafe { self.array.get_unchecked(idx) }
    }
}

pub struct A {
    pub start: usize,
    pub len: usize,
}

impl<T> std::ops::Index<A> for UnsafeFixArray<T> {
    type Output = [T];

    fn index(&self, rng: A) -> &Self::Output {
        unsafe {
            let ptr = self.array.get_unchecked(rng.start) as *const T;
            std::slice::from_raw_parts(ptr, rng.len)
        }
    }
}

impl<T> std::ops::Index<std::ops::Range<usize>> for UnsafeFixArray<T> {
    type Output = [T];

    fn index(&self, rng: std::ops::Range<usize>) -> &Self::Output {
        unsafe {
            let ptr = self.array.get_unchecked(rng.start) as *const T;
            std::slice::from_raw_parts(ptr, rng.len())
        }
    }
}

impl<T> From<Box<[T]>> for UnsafeFixArray<T> {
    fn from(array: Box<[T]>) -> Self {
        UnsafeFixArray { array: array }
    }
}

impl<T> From<Vec<T>> for UnsafeFixArray<T> {
    fn from(array: Vec<T>) -> Self {
        UnsafeFixArray {
            array: array.into_boxed_slice(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Resource {
    pub min: UnsafeFixArray<f64>,
    pub max: UnsafeFixArray<f64>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct MaintenanceMapping {
    pub resources: HashMap<RID, String>,
    pub interventions: HashMap<IID, String>,
    pub seasons: HashMap<SID, String>,
}

#[derive(Default, Debug)]
pub struct MaintenanceBuilder {
    ndays: Option<usize>,
    quantile: Option<f64>,
    alpha: Option<f64>,
    interventions: Option<Box<[Intervention]>>,
    resources: Option<Box<[Resource]>>,
    exclusions: Option<Box<[InterventionExclusions]>>,
    scenarios_number: Option<Vec<usize>>,
}

impl MaintenanceBuilder {
    fn new() -> Self {
        MaintenanceBuilder::default()
    }

    pub fn build(self) -> Maintenance {
        let quantile = self.quantile.unwrap();
        let scenarios_number = self.scenarios_number.unwrap();
        let nscenarios = *scenarios_number.iter().max().unwrap();
        let quantiles = scenarios_number
            .iter()
            .map(|&nb| ((nb as f64) * quantile).ceil() as usize - 1usize)
            .collect();
        Maintenance {
            ndays: self.ndays.unwrap(),
            quantile,
            quantiles,
            alpha: self.alpha.unwrap(),
            interventions: self.interventions.unwrap(),
            resources: self.resources.unwrap(),
            exclusions: self.exclusions.unwrap(),
            scenarios_number,
            nscenarios,
        }
    }

    pub fn set_ndays(self, ndays: usize) -> Self {
        MaintenanceBuilder {
            ndays: Some(ndays),
            quantile: self.quantile,
            alpha: self.alpha,
            interventions: self.interventions,
            resources: self.resources,
            exclusions: self.exclusions,
            scenarios_number: self.scenarios_number,
        }
    }

    pub fn set_quantile(self, quantile: f64) -> Self {
        MaintenanceBuilder {
            ndays: self.ndays,
            quantile: Some(quantile),
            alpha: self.alpha,
            interventions: self.interventions,
            resources: self.resources,
            exclusions: self.exclusions,
            scenarios_number: self.scenarios_number,
        }
    }

    pub fn set_alpha(self, alpha: f64) -> Self {
        MaintenanceBuilder {
            ndays: self.ndays,
            quantile: self.quantile,
            alpha: Some(alpha),
            interventions: self.interventions,
            resources: self.resources,
            exclusions: self.exclusions,
            scenarios_number: self.scenarios_number,
        }
    }

    pub fn set_interventions(self, interventions: Box<[Intervention]>) -> Self {
        MaintenanceBuilder {
            ndays: self.ndays,
            quantile: self.quantile,
            alpha: self.alpha,
            interventions: Some(interventions),
            resources: self.resources,
            exclusions: self.exclusions,
            scenarios_number: self.scenarios_number,
        }
    }

    pub fn set_resources(self, resources: Box<[Resource]>) -> Self {
        MaintenanceBuilder {
            ndays: self.ndays,
            quantile: self.quantile,
            alpha: self.alpha,
            interventions: self.interventions,
            resources: Some(resources),
            exclusions: self.exclusions,
            scenarios_number: self.scenarios_number,
        }
    }

    pub fn set_exclusions(self, exclusions: Box<[InterventionExclusions]>) -> Self {
        MaintenanceBuilder {
            ndays: self.ndays,
            quantile: self.quantile,
            alpha: self.alpha,
            interventions: self.interventions,
            resources: self.resources,
            exclusions: Some(exclusions),
            scenarios_number: self.scenarios_number,
        }
    }

    pub fn set_scenarios_number(self, scenarios_number: Vec<usize>) -> Self {
        MaintenanceBuilder {
            ndays: self.ndays,
            quantile: self.quantile,
            alpha: self.alpha,
            interventions: self.interventions,
            resources: self.resources,
            exclusions: self.exclusions,
            scenarios_number: Some(scenarios_number),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Maintenance {
    ndays: usize,
    quantile: f64,
    alpha: f64,
    interventions: Box<[Intervention]>,
    resources: Box<[Resource]>,
    exclusions: Box<[InterventionExclusions]>,
    scenarios_number: Vec<usize>, // expected values
    quantiles: Vec<usize>,        // expected values
    nscenarios: usize,
}

impl Maintenance {
    pub fn builder() -> MaintenanceBuilder {
        MaintenanceBuilder::new()
    }

    pub fn scenarios_number(&self) -> &[usize] {
        &self.scenarios_number
    }

    pub fn quantiles_by_period(&self, period: &Period) -> &[usize] {
        unsafe {
            let ptr = self.quantiles.get_unchecked(period.start().get()) as *const usize;
            std::slice::from_raw_parts(ptr, period.duration().get())
        }
    }

    pub fn scenarios_number_by_period(&self, period: &Period) -> &[usize] {
        unsafe {
            let ptr = self.scenarios_number.get_unchecked(period.start().get()) as *const usize;
            std::slice::from_raw_parts(ptr, period.duration().get())
        }
    }

    pub fn quantiles(&self) -> &[usize] {
        &self.quantiles
    }

    pub fn quantile(&self) -> f64 {
        self.quantile
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    pub fn exclusions(&self, iid: IID) -> &InterventionExclusions {
        unsafe { &self.exclusions.get_unchecked(iid.get()) }
    }

    pub fn nresources(&self) -> usize {
        self.resources.len()
    }

    pub fn resources(&self) -> &[Resource] {
        &self.resources[..]
    }

    pub fn ndays(&self) -> usize {
        self.ndays
    }

    pub fn ninterventions(&self) -> usize {
        self.interventions.len()
    }

    pub fn interventions_with_ids(&self) -> impl Iterator<Item = (IID, &Intervention)> {
        self.interventions
            .iter()
            .enumerate()
            .map(|(iid, int)| (IID::new(iid), int))
    }

    pub fn interventions(&self) -> impl Iterator<Item = &Intervention> {
        self.interventions.iter()
    }

    pub fn intervention(&self, iid: IID) -> &Intervention {
        unsafe { &self.interventions.get_unchecked(iid.get()) }
    }

    pub fn nscenarios(&self) -> usize {
        self.nscenarios
    }

    pub fn interventions_ids(&self) -> impl Iterator<Item = IID> {
        (0..self.interventions.len()).map(IID::new)
    }

    pub fn scenarios_ids(&self) -> impl Iterator<Item = SID> {
        (0..self.scenarios_number.len()).map(SID::new)
    }
}

pub struct Planning {
    pub interventions: Vec<(IID, Day)>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct MaintenanceOptimization {
    pub maintenance: Maintenance,
    pub mapping: MaintenanceMapping,
    pub computation_time: Duration,
}
