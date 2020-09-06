use crate::common::types::*;

#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub type Seasons = fixedbitset::FixedBitSet;

#[inline(always)]
pub fn intersect(lhs: &Seasons, rhs: &Seasons) -> bool {
    !lhs.as_slice()
        .iter()
        .zip(rhs.as_slice().iter())
        .all(|(x, y)| x & y == 0)
}

#[derive(Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct InterventionExclusions {
    pub exclusions: Box<[(IID, Seasons)]>,
}

impl InterventionExclusions {
    /// Test if a resource is excluded for a given season and some given resources.
    ///
    /// # Arugments
    /// * `sid` - Identifier of the season
    /// * `sid` - Identifiers of the interventions in increasing order
    ///
    /// # Example
    ///
    /// ```
    /// let x = 0u32;
    /// ```
    /// TODO(vincent): alias for Seasons
    pub fn is_excluded(
        &self,
        seasons: &Seasons,
        interventions: impl IntoIterator<Item = IID>,
    ) -> bool {
        if self.exclusions.is_empty() {
            return false;
        }
        let mut pos = 0usize;
        for iid in interventions.into_iter() {
            if pos >= self.exclusions.len() {
                return false;
            }
            let slice = &self.exclusions[pos..];
            if let Ok(idx) = slice.binary_search_by(|(ex_iid, _)| ex_iid.cmp(&iid)) {
                pos = idx;
                if !intersect(&unsafe { self.exclusions.get_unchecked(idx) }.1, &seasons) {
                    return true;
                }
            }
        }
        false
    }

    pub fn excluded_interventions(&self, seasons: &Seasons) -> Vec<IID> {
        self.exclusions
            .iter()
            .filter(|(_, ex_seasons)| ex_seasons.is_disjoint(&seasons))
            .map(|(iid, _)| *iid)
            .collect()
    }

    pub fn possible_seasons(&self, interventions: &[Seasons]) -> Seasons {
        let mut possible_seasons = Seasons::with_capacity(interventions[0].len());
        possible_seasons.insert_range(..);
        // use tuple instead of a struct
        for (iid, seasons) in self.exclusions.iter() {
            for sid in seasons.intersection(&interventions[iid.get()]) {
                possible_seasons.set(sid, false);
            }
            if possible_seasons.ones().next().is_some() {
                break;
            }
        }
        possible_seasons
    }
}
