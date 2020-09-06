use crate::common::types::*;

// change periods slice by period
// multiply by scenarios for the indexes
#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Risks {
    nscenarios: usize,
    periods_slice: Box<[usize]>,
    risks: Box<[f64]>,
    // use ref instead
    //periods_number: Box<[usize]>,
    summed_risks: Box<[f64]>,
}

impl Risks {
    pub fn builder() -> RiskBuilder {
        RiskBuilder
    }

    // for the whole period
    pub fn values(&self, day: Day) -> &[f64] {
        let idx = day.get();
        unsafe {
            let start = self.periods_slice.get_unchecked(idx);
            let end = self.periods_slice.get_unchecked(idx + 1);
            let n = end - start + 1;
            let ptr = self.risks.get_unchecked(*start) as *const f64;

            std::slice::from_raw_parts(ptr, n)
        }
        //let bounds = &self.periods_slice[idx..(idx + 2)];
        //&self.risks[bounds[0]..bounds[1]]
    }

    /// sum of risk for each day
    pub fn summed_values(&self, day: Day) -> &[f64] {
        let idx = day.get();
        // BOUNDS are false now
        // can't divie by nscernarios
        // because nscenarios depends of the day
        unsafe {
            let start = self.periods_slice.get_unchecked(idx) / self.nscenarios;
            let end = self.periods_slice.get_unchecked(idx + 1) / self.nscenarios;
            let n = end - start + 1;
            let ptr = self.summed_risks.get_unchecked(start) as *const f64;

            std::slice::from_raw_parts(ptr, n)
        }
        //let bounds = &self.periods_slice[idx..(idx + 2)];
        //&self.summed_risks[(bounds[0] / self.nscenarios)..(bounds[1] / self.nscenarios)]
    }

    ///// sum of risk for each day
    //pub fn summed_values_by_period(&self) -> Chunks<&[f64]> {
    //// TODO(vincent): create dedicated iterator
    //let idx = day.get();
    //// BOUNDS are false now
    //// can't divie by nscernarios
    //// because nscenarios depends of the day
    //let bounds = &self.periods_slice[idx..(idx + 2)];
    //&self.summed_risks[(bounds[0] / self.nscenarios)..(bounds[1] / self.nscenarios)]
    //}

    ///// Chunck of scenarios size
    //pub fn values_by_day(&self, day: Day) -> Chunks<f64> {
    //let idx = day.get();
    //let bounds = &self.periods_slice[idx..(idx + 2)];
    //let risks = &self.risks[bounds[0]..bounds[1]];
    //risks.chunks(self.nscenarios)
    //}
}

pub struct RiskBuilder;
pub struct RiskBuilderStep2 {
    nscenarios: usize,
}
pub struct RiskBuilderStep3 {
    nscenarios: usize,
    periods_slice: Box<[usize]>,
}
pub struct RiskBuilderStepFinal {
    nscenarios: usize,
    periods_slice: Box<[usize]>,
    risks: Box<[f64]>,
}

impl RiskBuilder {
    pub fn set_nscenarios(self, nscernarios: usize) -> RiskBuilderStep2 {
        RiskBuilderStep2 {
            nscenarios: nscernarios,
        }
    }
}

impl RiskBuilderStep2 {
    pub fn set_periods(self, periods_slice: Box<[usize]>) -> RiskBuilderStep3 {
        RiskBuilderStep3 {
            nscenarios: self.nscenarios,
            periods_slice,
        }
    }
}

impl RiskBuilderStep3 {
    pub fn set_risks(self, risks: Box<[f64]>) -> RiskBuilderStepFinal {
        RiskBuilderStepFinal {
            nscenarios: self.nscenarios,
            periods_slice: self.periods_slice,
            risks,
        }
    }
}
impl RiskBuilderStepFinal {
    pub fn build(self) -> Risks {
        let risks = self.risks;
        let summed_risks = risks
            .chunks(self.nscenarios)
            .map(|risks| risks.iter().sum())
            .collect();
        Risks {
            nscenarios: self.nscenarios,
            periods_slice: self.periods_slice,
            risks,
            summed_risks,
        }
    }
}
