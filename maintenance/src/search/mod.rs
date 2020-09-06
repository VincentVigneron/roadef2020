use crate::common::exclusion::*;
use crate::common::intervention::*;
use crate::common::types::*;
use crate::common::{Maintenance, Planning};

mod state;

use self::state::*;

pub trait Search<'maintenance> {
    fn build(maintenance: &'maintenance Maintenance) -> Self;
    fn search(&self) -> Planning;
    fn search_from(&self, planning: &Planning) -> Planning;
}

pub struct LocalSearch<'maintenance> {
    maintenance: &'maintenance Maintenance,
    state: SearchState<'maintenance>,
}

// TODO(vincent): support for cumuled_workloads
impl<'maintenance> LocalSearch<'maintenance> {
    pub fn new(maintenance: &'maintenance Maintenance) -> LocalSearch<'maintenance> {
        LocalSearch {
            maintenance,
            state: SearchState {
                interventions: vec![None; maintenance.ninterventions()].into_boxed_slice(),
                seasons: vec![None; maintenance.ninterventions()].into_boxed_slice(),
                unplanned: maintenance.interventions_ids().collect(),
                planned: vec![],
                workloads: WorkloadsState {
                    workloads: vec![
                        vec![0.0f64; maintenance.ndays()].into_boxed_slice();
                        maintenance.nresources()
                    ]
                    .into_boxed_slice(),
                    //cumuled_workloads: vec![
                    //vec![0.0f64; maintenance.ndays()].into_boxed_slice();
                    //maintenance.nresources()
                    //]
                    //.into_boxed_slice(),
                },
                cost: CostState {
                    risks: vec![0.0f64; maintenance.ndays() * maintenance.nscenarios()]
                        .into_boxed_slice(),
                    summed_risks: vec![0.0f64; maintenance.ndays()].into_boxed_slice(),
                    mean_risks: vec![0.0f64; maintenance.ndays()].into_boxed_slice(),
                    summed_mean_risks: 0.0f64,
                    quantile_risks: vec![0.0f64; maintenance.ndays()].into_boxed_slice(),
                    excess_risks: vec![0.0f64; maintenance.ndays()].into_boxed_slice(),
                    summed_excess: 0.0f64,
                    cost: 0.0f64,
                    nscenarios: maintenance.nscenarios(),
                },
            },
        }
    }

    pub fn init_(&mut self) {
        for iid in self.maintenance.interventions_ids() {
            let day = self
                .maintenance
                .intervention(iid)
                .days()
                .find(|&day| self.schedulable(iid, day));
            if let Some(day) = day {
                self.schedule(iid, day);
            }
        }
    }

    pub fn init(&mut self) {
        let mut interventions: Vec<(IID, (Day, f64))> = self
            .maintenance
            .interventions()
            .map(|i| {
                (
                    i.periods().map(Period::duration).max().unwrap(),
                    i.periods()
                        .map(|p| i.summed_risks(p.start()).iter().sum::<f64>())
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                )
            })
            .enumerate()
            .map(|(iid, k)| (IID::new(iid), k))
            .collect();
        interventions.sort_by(|a, b| (b.1, b.0).partial_cmp(&(a.1, a.0)).unwrap());

        for iid in interventions.iter().map(|x| x.0) {
            let days = {
                let intervention = self.maintenance.intervention(iid);
                let mut days: Vec<(Day, (Day, f64))> = intervention
                    .periods()
                    .map(|p| {
                        (
                            p.start(),
                            (
                                p.duration(),
                                intervention.summed_risks(p.start()).iter().sum::<f64>(),
                            ),
                        )
                    })
                    .collect();
                days.sort_by(|a, b| (b.1, b.0).partial_cmp(&(a.1, a.0)).unwrap());
                days
            };
            let day = days
                .iter()
                .map(|x| x.0)
                .find(|&day| self.schedulable(iid, day));
            if let Some(day) = day {
                self.schedule(iid, day);
            }
        }
    }

    fn schedulable(&self, iid: IID, day: Day) -> bool {
        let intervention = self.maintenance.intervention(iid);
        let period = intervention.period(day);
        self.check_exclusion(
            &period,
            intervention.seasons(day),
            self.maintenance.exclusions(iid),
        ) && self.state.workloads.check_adding(
            &period,
            intervention.workloads(period.start()),
            self.maintenance.resources(),
        )
    }

    fn check_exclusion(
        &self,
        period: &Period,
        seasons: &Seasons,
        exclusions: &InterventionExclusions,
    ) -> bool {
        let iids = self
            .state
            .interventions
            .iter()
            .enumerate()
            .filter_map(|(iid, p)| match p.as_ref() {
                Some(p) if p.intersect(&period) => Some(iid),
                _ => None,
            })
            .filter_map(
                |iid| match unsafe { self.state.seasons.get_unchecked(iid) } {
                    Some(s) if intersect(s, &seasons) => Some(IID::new(iid)),
                    _ => None,
                },
            );
        !exclusions.is_excluded(&seasons, iids)
    }

    fn schedule(&mut self, iid: IID, new_day: Day) {
        let intervention = self.maintenance.intervention(iid);
        let new_period = *intervention.period(new_day);
        match unsafe { self.state.interventions.get_unchecked(iid.get()) } {
            Some(ref cur_period) if *cur_period != new_period => {
                //unimplemented!()
                //decrease_risks();
                //increase_risks().
                //...
            }
            None => {
                self.state.unplanned.remove_item(&iid);
                self.state.planned.push(iid);
                self.increase_workloads(&new_period, intervention);
                self.increase_risks(&new_period, intervention);
                unsafe {
                    *self.state.interventions.get_unchecked_mut(iid.get()) = Some(new_period);
                    *self.state.seasons.get_unchecked_mut(iid.get()) =
                        Some(intervention.seasons(new_day));
                }
            }
            _ => {
                // no change
            }
        }
    }

    fn increase_workloads(&mut self, period: &Period, intervention: &Intervention) {
        self.state
            .workloads
            .increase_workloads(period, intervention.workloads(period.start()));
    }

    fn increase_risks(&mut self, period: &Period, intervention: &Intervention) {
        self.state
            .risk_incrementer()
            .update_risks(period, intervention.period_risks(period.start()))
            .update_mean(
                period,
                intervention.summed_risks(period.start()),
                &self.maintenance.scenarios_number_by_period(period),
            )
            .update_quantile(
                period,
                &self.maintenance.scenarios_number_by_period(period),
                &self.maintenance.quantiles_by_period(period),
            )
            .update_excess(period)
            .update_cost(self.maintenance.ndays(), self.maintenance.alpha());
    }

    pub fn current_planning(&self) -> Planning {
        let interventions = self
            .state
            .interventions
            .iter()
            .enumerate()
            .filter(|(_, i)| i.is_some())
            .map(|(iid, i)| (IID::new(iid), i.unwrap().start()))
            .collect();
        Planning { interventions }
    }
}

impl<'maintenance> Search<'maintenance> for LocalSearch<'maintenance> {
    fn build(maintenance: &'maintenance Maintenance) -> LocalSearch<'maintenance> {
        Self::new(maintenance)
    }

    fn search(&self) -> Planning {
        Planning {
            interventions: vec![],
        }
    }

    fn search_from(&self, _planning: &Planning) -> Planning {
        Planning {
            interventions: vec![],
        }
    }
}
