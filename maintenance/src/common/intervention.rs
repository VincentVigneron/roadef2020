use common::exclusion::*;
use common::risks::*;
use common::types::*;

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Workload {
    rid: RID,
    workloads: Box<[f64]>,
    cumuled_workloads: Box<[f64]>,
    total_workloads: f64,
}

impl Workload {
    pub fn new(rid: RID, workloads: Box<[f64]>) -> Self {
        let total_workloads = workloads.iter().sum();
        let cumuled_workloads = workloads
            .iter()
            .scan(0.0f64, |state, &wl| {
                *state += wl;
                Some(*state)
            })
            .collect();
        Workload {
            rid,
            workloads,
            cumuled_workloads,
            total_workloads,
        }
    }

    pub fn rid(&self) -> RID {
        self.rid
    }

    pub fn workloads(&self) -> &[f64] {
        &self.workloads
    }

    pub fn cumuled_workloads(&self) -> &[f64] {
        &self.cumuled_workloads
    }

    pub fn total_workloads(&self) -> f64 {
        self.total_workloads
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Intervention {
    /// lastest starting_day for the intervention
    latest_start: Day,
    /// Period corresponding to a given starting day
    /// Access through PID
    periods: Box<[Period]>,
    /// Season covered by a given period
    /// Access through PID
    seasons: Box<[Seasons]>,
    /// risks associated to a given period
    /// Access through PID
    risks: Risks,
    /// List of possible resources
    /// p0,r0,p0r1,..p0rn,p1r0,....
    workloads: Box<[Workload]>,
    resources: Box<[RID]>,
}

impl Intervention {
    pub fn latest_start(&self) -> Day {
        self.latest_start
    }

    pub fn is_day_compatible(&self, day: Day) -> bool {
        day <= self.latest_start
    }

    pub fn days(&self) -> impl Iterator<Item = Day> {
        (0..=self.latest_start.get()).map(Day::new)
    }

    pub fn periods(&self) -> impl Iterator<Item = &Period> {
        self.periods.iter()
    }

    pub fn period(&self, day: Day) -> &Period {
        unsafe { &self.periods.get_unchecked(day.get()) }
    }

    //// nscenarios chunks of size of the period
    //pub fn risks(&self, day: Day) -> Chunks<f64> {
    //self.risks.values_by_day(day)
    //}

    // sum by day
    pub fn summed_risks(&self, day: Day) -> &[f64] {
        self.risks.summed_values(day)
    }

    //pub fn summed_risks_by_period(&self, day: Day) -> Chunk<&[f64]> {
    //self.risks.summed_values(day)
    //}

    pub fn period_risks(&self, day: Day) -> &[f64] {
        self.risks.values(day)
    }

    pub fn seasons(&self, day: Day) -> &Seasons {
        unsafe { &self.seasons.get_unchecked(day.get()) }
    }

    pub fn nresources(&self) -> usize {
        self.resources.len()
    }

    pub fn workloads(&self, day: Day) -> &[Workload] {
        let start = day.get() * self.nresources();
        //let end = (day.get() + 1) * self.nresources();
        unsafe {
            let ptr = self.workloads.get_unchecked(start) as *const Workload;
            std::slice::from_raw_parts(ptr, self.nresources())
        }
        //&self.workloads[start..end]
    }

    pub fn builder() -> InterventionBuilder {
        InterventionBuilder::new()
    }
}

#[derive(Default)]
pub struct InterventionBuilder {
    latest_start: Option<Day>,
    periods: Option<Box<[Period]>>,
    seasons: Option<Box<[Seasons]>>,
    risks: Option<Risks>,
    workloads: Option<Box<[Workload]>>,
    resources: Option<Box<[RID]>>,
}
impl InterventionBuilder {
    fn new() -> Self {
        InterventionBuilder::default()
    }

    pub fn build(self) -> Intervention {
        Intervention {
            latest_start: self.latest_start.unwrap(),
            periods: self.periods.unwrap(),
            seasons: self.seasons.unwrap(),
            risks: self.risks.unwrap(),
            workloads: self.workloads.unwrap(),
            resources: self.resources.unwrap(),
        }
    }

    pub fn set_latest_start<D: Into<Day>>(self, day: D) -> Self {
        InterventionBuilder {
            latest_start: Some(day.into()),
            periods: self.periods,
            seasons: self.seasons,
            risks: self.risks,
            workloads: self.workloads,
            resources: self.resources,
        }
    }

    pub fn set_periods(self, periods: Box<[Period]>) -> Self {
        InterventionBuilder {
            latest_start: self.latest_start,
            periods: Some(periods),
            seasons: self.seasons,
            risks: self.risks,
            workloads: self.workloads,
            resources: self.resources,
        }
    }

    pub fn set_seasons(self, seasons: Box<[Seasons]>) -> Self {
        InterventionBuilder {
            latest_start: self.latest_start,
            periods: self.periods,
            seasons: Some(seasons),
            risks: self.risks,
            workloads: self.workloads,
            resources: self.resources,
        }
    }

    pub fn set_risks(self, risks: Risks) -> Self {
        InterventionBuilder {
            latest_start: self.latest_start,
            periods: self.periods,
            seasons: self.seasons,
            risks: Some(risks),
            workloads: self.workloads,
            resources: self.resources,
        }
    }

    pub fn set_resources(self, resources: Box<[RID]>) -> Self {
        InterventionBuilder {
            latest_start: self.latest_start,
            periods: self.periods,
            seasons: self.seasons,
            risks: self.risks,
            workloads: self.workloads,
            resources: Some(resources),
        }
    }

    pub fn set_workloads(self, workloads: Box<[Workload]>) -> Self {
        InterventionBuilder {
            latest_start: self.latest_start,
            periods: self.periods,
            seasons: self.seasons,
            risks: self.risks,
            workloads: Some(workloads),
            resources: self.resources,
        }
    }
}
