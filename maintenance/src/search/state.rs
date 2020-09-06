use crate::common::intervention::*;
use crate::common::*;
use crate::utils::*;
use crate::{Period, Seasons, IID /*, RID*/};
// TODO(vincent): Remove derive default
#[derive(Default)]
#[allow(dead_code)]
pub struct SearchState<'maintenance> {
    /// Number of interventions
    pub interventions: Box<[Option<Period>]>,
    /// Number of interventions
    pub seasons: Box<[Option<&'maintenance Seasons>]>,
    pub workloads: WorkloadsState,
    pub cost: CostState,
    pub unplanned: Vec<IID>,
    pub planned: Vec<IID>,
}

#[derive(Default)]
pub struct WorkloadsState {
    pub workloads: Box<[Box<[f64]>]>,
    // buffer
    //
    // maybre useless
    // cumuled requirees to update every cumuled value after the period
    // by the change at last index provided by the cumuled...
    //cumuled_workloads: Box<[Box<[f64]>]>,
}

impl WorkloadsState {
    pub fn increase_workloads(&mut self, period: &Period, workloads: &[Workload]) {
        let begin = period.start().get();
        let end = period.end_exclusive().get();
        workloads.iter().for_each(|wl| {
            add_vec_in_place(
                &mut self.workloads[wl.rid().get()][begin..end],
                wl.workloads(),
            )
        });
    }
}

#[allow(dead_code)]
enum WorkloadStatus {
    Ok,
    UnderMin,
    OverMax,
}

impl WorkloadsState {
    pub fn check_adding(
        &self,
        period: &Period,
        workloads: &[Workload],
        resource_bounds: &[Resource],
    ) -> bool {
        use fast_floats::Fast;
        let begin = period.start().get();
        //let duration = period.duration().get();
        //let end = period.end_exclusive().get();
        //let cumuled = workloads.iter()
        //.map(|wl| (wl.total_workloads(), &self.cumuled_workloads[wl.rid().get()]))
        //.map(|(_,cumuled)| cumuled[cumuled.len() - 1] - cumuled[0])
        //.map(|(total, cumuled)| total + cumuled)
        //.infboud
        //unsafe {
        //workloads
        //.iter()
        //.map(|wl| {
        //wl.workloads().iter().zip(
        //self.workloads.get_unchecked(wl.rid().get())[begin..end]
        //.iter()
        //.zip(
        //resource_bounds.get_unchecked(wl.rid().get()).max[begin..end]
        //.iter(),
        //),
        //)
        ////.skip(begin)
        ////.take(duration)
        //})
        //.all(|mut wl| wl.all(|(&wl, (&curr_wl, &max))| wl + curr_wl <= max))
        //}
        //

        workloads
            .iter()
            .map(|wl| {
                let workloads = wl.workloads();
                let n = workloads.len();
                let cur_wl = unsafe {
                    let ptr = self
                        .workloads
                        .get_unchecked(wl.rid().get())
                        .get_unchecked(begin) as *const f64;
                    std::slice::from_raw_parts(ptr, n)
                };
                let maxs = unsafe {
                    //let ptr =
                    &resource_bounds.get_unchecked(wl.rid().get()).max[crate::common::A {
                        start: begin,
                        len: n,
                    }]
                    //.get_unchecked(begin) as *const f64;
                    //std::slice::from_raw_parts(ptr, n)
                };
                workloads.iter().zip(cur_wl.iter().zip(maxs.iter()))
            })
            .all(|mut wl| wl.all(|(&wl, (&curr_wl, &max))| wl + curr_wl <= max))
        //for wl in workloads {
        //let workloads = wl.workloads();
        //let cur_wl = unsafe { &self.workloads.get_unchecked(wl.rid().get()) };
        //let maxs = unsafe { &resource_bounds.get_unchecked(wl.rid().get()).max };
        //for idx in 0..workloads.len() {
        //unsafe {
        //if workloads.get_unchecked(idx) + cur_wl.get_unchecked(idx + begin)
        //> *maxs.get_unchecked(idx + begin)
        //{
        //return false;
        //}
        //}
        //}
        //}
        //false
    }
}

#[derive(Default)]
pub struct CostState {
    /// number of scenarios
    pub nscenarios: usize,
    pub risks: Box<[f64]>,
    pub summed_risks: Box<[f64]>,
    pub mean_risks: Box<[f64]>,
    pub summed_mean_risks: f64,
    pub quantile_risks: Box<[f64]>,
    pub excess_risks: Box<[f64]>,
    pub summed_excess: f64,
    pub cost: f64,
}

// create state machine for adding risk on a given period
// The state machine is here to enforce the order of updates
// to avoid updating excess or cost before updating the median
// and the risk..etc
// risks -> mean     -> excess -> cost
//       -> quantile
pub struct RisksIncrementerAddRisks<'state> {
    state: &'state mut CostState,
}

pub struct RisksIncrementerUpdateMean<'state> {
    state: &'state mut CostState,
}

pub struct RisksIncrementerUpdateQuantile<'state> {
    state: &'state mut CostState,
}

pub struct RisksIncrementerUpdateExcess<'state> {
    state: &'state mut CostState,
}

pub struct RisksIncrementerUpdateCost<'state> {
    state: &'state mut CostState,
}

impl<'state> RisksIncrementerAddRisks<'state> {
    #[inline]
    pub fn update_risks(
        self,
        period: &Period,
        risks: &[f64],
    ) -> RisksIncrementerUpdateMean<'state> {
        let begin = period.start().get() * self.state.nscenarios;
        //let end = period.end_exclusive().get() * self.state.nscenarios;
        //add_vec_in_place(&mut self.state.risks[begin..end], risks);

        let mut st_risks = unsafe {
            let ptr = self.state.risks.get_unchecked_mut(begin);
            std::slice::from_raw_parts_mut(ptr, risks.len())
        }; //(&mut self.state.quantile_risks[begin..end]).iter_mut();
        add_vec_in_place(&mut st_risks, risks);

        RisksIncrementerUpdateMean { state: self.state }
    }
}

impl<'state> RisksIncrementerUpdateMean<'state> {
    #[inline]
    pub fn update_mean(
        self,
        period: &Period,
        summed_risks: &[f64],
        scenarios_number: &[usize],
    ) -> RisksIncrementerUpdateQuantile<'state> {
        let begin = period.start().get();
        //let end = period.end_exclusive().get();
        let mut st_summed_risks = unsafe {
            let ptr = self.state.summed_risks.get_unchecked_mut(begin);
            std::slice::from_raw_parts_mut(ptr, summed_risks.len())
        };
        add_vec_in_place(&mut st_summed_risks, summed_risks);
        let mut st_mean_risks = unsafe {
            let ptr = self.state.mean_risks.get_unchecked_mut(begin);
            std::slice::from_raw_parts_mut(ptr, summed_risks.len())
        };
        //let summed_mean_on_period: f64 = self.state.mean_risks[begin..end].iter().sum();
        let summed_mean_on_period: f64 = st_mean_risks.iter().sum();
        mean_vec(
            &mut st_mean_risks,
            &st_summed_risks,
            &scenarios_number,
            //&mut self.state.mean_risks[begin..end],
            //&self.state.summed_risks[begin..end],
            //&scenarios_number,
        );
        let summed_mean_on_period = st_mean_risks.iter().sum::<f64>() - summed_mean_on_period;
        //self.state.mean_risks[begin..end].iter().sum::<f64>() - summed_mean_on_period;
        self.state.summed_mean_risks += summed_mean_on_period;
        RisksIncrementerUpdateQuantile { state: self.state }
    }
}

impl<'state> RisksIncrementerUpdateQuantile<'state> {
    #[inline]
    pub fn update_quantile(
        self,
        period: &Period,
        scenarios_number: &[usize],
        quantiles: &[usize],
    ) -> RisksIncrementerUpdateExcess<'state> {
        let begin = period.start().get();

        let sn = scenarios_number.iter();
        let qu = quantiles.iter();
        let len = quantiles.len();
        let ri = unsafe {
            let ptr = self
                .state
                .quantile_risks
                .get_unchecked_mut(begin * self.state.nscenarios);
            std::slice::from_raw_parts_mut(ptr, len * self.state.nscenarios)
                .chunks(self.state.nscenarios)
        };
        // (&self.state.risks[begin * self.state.nscenarios..end * self.state.nscenarios])
        let qu_ri = unsafe {
            let ptr = self.state.quantile_risks.get_unchecked_mut(begin);
            std::slice::from_raw_parts_mut(ptr, len)
        }; //(&mut self.state.quantile_risks[begin..end]).iter_mut();
        for ((&sn, &qu), (ri, qu_ri)) in sn.zip(qu).zip(ri.zip(qu_ri)) {
            *qu_ri = nth_element(&ri[..sn], qu).unwrap();
            //*qu_ri = dummy_quantile(&ri[..sn], qu).unwrap();
        }
        RisksIncrementerUpdateExcess { state: self.state }
    }
}

impl<'state> RisksIncrementerUpdateExcess<'state> {
    #[inline]
    pub fn update_excess(self, period: &Period) -> RisksIncrementerUpdateCost<'state> {
        let begin = period.start().get();
        let end = period.end_exclusive().get();
        let sum_exsc: f64 = { (&self.state.excess_risks[begin..end]).iter().sum() };
        let means = (&self.state.mean_risks[begin..end]).iter();
        let qu_ri = (&self.state.quantile_risks[begin..end]).iter();
        let qu_ex = (&mut self.state.excess_risks[begin..end]).iter_mut();
        for (ex, (&mean, &quantile)) in qu_ex.zip(means.zip(qu_ri)) {
            *ex = if quantile < mean {
                0.0f64
            } else {
                quantile - mean
            }
        }
        let summed_excs_on_period =
            self.state.excess_risks[begin..end].iter().sum::<f64>() - sum_exsc;
        self.state.summed_excess += summed_excs_on_period;
        RisksIncrementerUpdateCost { state: self.state }
    }
}

impl<'state> RisksIncrementerUpdateCost<'state> {
    #[inline]
    pub fn update_cost(self, ndays: usize, alpha: f64) {
        let ndays = ndays as f64;
        let obj1 = (self.state.summed_mean_risks as f64) / ndays;
        let obj2 = (self.state.summed_excess as f64) / ndays;
        self.state.cost = alpha * obj1 + (1.0f64 - alpha) * obj2;
    }
}

impl<'maintenance> SearchState<'maintenance> {
    // -> (WorkloadIncrementer, RiskIncrementer)
    // maybe 2 functions because borrow mut won't be possilb
    #[inline]
    pub fn risk_incrementer(&mut self) -> RisksIncrementerAddRisks<'_> {
        RisksIncrementerAddRisks {
            state: &mut self.cost,
        }
    }

    //fn seasons(&self, iid: IID) -> Option<&Seasons> {
    //*&self.seasons[iid.get()]
    //}

    //fn workloads(&self, rid: RID, period: &Period) -> &[f64] {
    //&self.workloads[rid.get()][period.start().get()..period.end_exclusive().get()]
    //}

    //fn workloads_mut(&mut self, rid: RID, period: &Period) -> &mut [f64] {
    //&mut self.workloads[rid.get()][period.start().get()..period.end_exclusive().get()]
    //}

    //fn cumuled_workloads(&self, rid: RID, period: &Period) -> &[f64] {
    //&self.cumuled_workloads[rid.get()][period.start().get()..period.end_exclusive().get()]
    //}

    //fn cumuled_workloads_mut(&mut self, rid: RID, period: &Period) -> &mut [f64] {
    //&mut self.cumuled_workloads[rid.get()][period.start().get()..period.end_exclusive().get()]
    //}

    //fn risks_by_period(&self, period: &Period) -> &[f64] {
    //let begin = period.start().get() * self.nscenarios;
    //let end = period.end_exclusive().get() * self.nscenarios;
    //&self.risks[begin..end]
    //}

    //fn risks_by_period_mut(&mut self, period: &Period) -> &mut [f64] {
    //let begin = period.start().get() * self.nscenarios;
    //let end = period.end_exclusive().get() * self.nscenarios;
    //&mut self.risks[begin..end]
    //}

    //fn summed_risks_by_period(&self, period: &Period) -> &[f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&self.summed_risks[begin..end]
    //}

    //fn summed_risks_by_period_mut(&mut self, period: &Period) -> &mut [f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&mut self.summed_risks[begin..end]
    //}

    //fn mean_risks_by_period(&self, period: &Period) -> &[f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&self.mean_risks[begin..end]
    //}

    //fn mean_risks_by_period_mut(&mut self, period: &Period) -> &mut [f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&mut self.mean_risks[begin..end]
    //}

    //fn quantile_risks_by_period(&self, period: &Period) -> &[f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&self.quantile_risks[begin..end]
    //}

    //fn quantile_risks_by_period_mut(&mut self, period: &Period) -> &mut [f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&mut self.quantile_risks[begin..end]
    //}

    //fn excess_risks_by_period(&self, period: &Period) -> &[f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&self.excess_risks[begin..end]
    //}

    //fn excess_risks_by_period_mut(&mut self, period: &Period) -> &mut [f64] {
    //let begin = period.start().get();
    //let end = period.end_exclusive().get();
    //&mut self.excess_risks[begin..end]
    //}
    //fn risks(&self, sid: SID, period: &Period) -> &[f64] {
    //&self.risks[sid.get()][period.start().get()..period.end_exclusive().get()]
    //}

    //fn risks_mut(&mut self, sid: SID, period: &Period) -> &mut [f64] {
    //&mut self.risks[sid.get()][period.start().get()..period.end_exclusive().get()]
    //}
}
