use std::ops::{Add, Sub};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Season identifier
pub struct SID(usize);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Day identifier
pub struct Day(usize);

/// Resource identifier
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct RID(usize);

/// Day identifier for a given period
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct PID(usize);

/// Intervention identifier for a given period
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct IID(usize);

/// Resource identifier for a given period
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct PRID(usize);

macro_rules! maintenance_identifier {
    ($type:ident) => {
        impl $type {
            pub fn new(id: usize) -> Self {
                $type(id)
            }

            pub fn get(&self) -> usize {
                let $type(id) = *self;
                id
            }
        }
    };
}

maintenance_identifier!(Day);
maintenance_identifier!(RID);
maintenance_identifier!(SID);
maintenance_identifier!(PID);
maintenance_identifier!(IID);
maintenance_identifier!(PRID);

impl Sub for Day {
    type Output = Day;
    fn sub(self, other: Day) -> Day {
        let Day(me) = self;
        let Day(other) = other;
        Day(me - other)
    }
}
impl Add for Day {
    type Output = Day;
    fn add(self, other: Day) -> Day {
        let Day(me) = self;
        let Day(other) = other;
        Day(me + other)
    }
}

/// Represents non empty period
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Period {
    start: Day,
    duration: Day,
}

impl Period {
    pub fn new(first: Day, duration: Day) -> Option<Self> {
        match duration {
            Day(0) => None,
            _ => Some(Period {
                start: first,
                duration,
            }),
        }
    }

    #[inline(always)]
    pub fn duration(&self) -> Day {
        self.duration
    }

    #[inline(always)]
    pub fn days(&self) -> (Day, Day) {
        (self.start, self.end())
    }

    #[inline(always)]
    pub fn days_exclusive(&self) -> (Day, Day) {
        (self.start, self.end_exclusive())
    }

    #[inline(always)]
    pub fn start(&self) -> Day {
        self.start
    }

    #[inline(always)]
    pub fn end(&self) -> Day {
        self.start + self.duration - Day(1)
    }

    #[inline(always)]
    pub fn end_exclusive(&self) -> Day {
        self.start + self.duration
    }

    #[inline(always)]
    pub fn contains(&self, other: &Period) -> bool {
        self.start <= other.start && self.end_exclusive() >= other.end_exclusive()
    }

    #[inline(always)]
    pub fn intersect(&self, other: &Period) -> bool {
        (other.start() > self.end()) || (self.start() > other.end())
    }

    pub fn intersection(&self, other: &Period) -> Option<Period> {
        if (other.start() > self.end()) || (self.start() > other.end()) {
            None
        } else {
            let start = std::cmp::max(other.start(), self.start());
            let end = std::cmp::min(other.end_exclusive(), self.end_exclusive());
            Some(Period {
                start,
                duration: end - start,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    mod period {
        #[test]
        fn new_empty() {
            for start in 0..=100 {
                let p = ::Period::new(::Day::new(start), ::Day::new(0));
                assert!(p.is_none());
            }
        }

        #[test]
        fn is_not_empty() {
            for start in 0..=100 {
                for duration in 1..=100 {
                    let p = ::Period::new(::Day::new(start), ::Day::new(duration));
                    assert!(p.is_some())
                }
            }
        }

        #[test]
        fn bounds() {
            for start in 0..=100 {
                for duration in 1..=100 {
                    let p = ::Period::new(::Day::new(start), ::Day::new(duration)).unwrap();
                    assert_eq!(p.start(), ::Day::new(start));
                    assert_eq!(p.end_exclusive(), ::Day::new(start + duration));
                    assert_eq!(
                        p.days_exclusive(),
                        (::Day::new(start), ::Day::new(start + duration))
                    );
                    assert_eq!(p.end(), ::Day::new(start + duration - 1));
                    assert_eq!(
                        p.days(),
                        (::Day::new(start), ::Day::new(start + duration - 1))
                    );
                }
            }
        }

        #[test]
        pub fn contains() -> Result<(), String> {
            let periods = [
                // starts at 0
                ("Allen::starts", [0, 3], [0, 1], true),
                ("Allen::finishes", [0, 3], [1, 3], true),
                ("Allen::During", [0, 3], [1, 2], true),
                ("Allen::equals", [0, 3], [0, 3], true),
                ("Allen::overlaps_xy", [0, 3], [2, 4], false),
                ("Allen::meets_xy", [0, 3], [4, 6], false),
                ("Allen::before_xy", [0, 3], [5, 6], false),
                // general
                ("Allen::starts", [2, 5], [2, 3], true),
                ("Allen::finishes", [2, 5], [3, 5], true),
                ("Allen::During", [2, 5], [3, 4], true),
                ("Allen::equals", [2, 5], [2, 5], true),
                ("Allen::overlaps_xy", [2, 5], [4, 7], false),
                ("Allen::meets_xy", [2, 5], [6, 7], false),
                ("Allen::before_xy", [2, 5], [7, 7], false),
                ("Allen::overlaps_yx", [2, 5], [0, 3], false),
                ("Allen::meets_yx", [2, 5], [0, 1], false),
                ("Allen::before_yx", [2, 5], [0, 0], false),
            ];
            for (code, x, y, expected) in periods.iter() {
                let x = ::Period::new(::Day::new(x[0]), ::Day::new(x[1] - x[0] + 1)).unwrap();
                let y = ::Period::new(::Day::new(y[0]), ::Day::new(y[1] - y[0] + 1)).unwrap();
                if x.contains(&y) != *expected {
                    return Err(String::from(*code));
                }
            }
            Ok(())
        }

        #[test]
        pub fn intersect() -> Result<(), String> {
            let periods = [
                // starts at 0
                ("Allen::starts", [0, 3], [0, 1], Some([0, 1])),
                ("Allen::finishes", [0, 3], [1, 3], Some([1, 3])),
                ("Allen::During", [0, 3], [1, 2], Some([1, 2])),
                ("Allen::equals", [0, 3], [0, 3], Some([0, 3])),
                ("Allen::overlaps_xy", [0, 3], [2, 4], Some([2, 3])),
                ("Allen::meets_xy", [0, 3], [4, 6], None),
                ("Allen::before_xy", [0, 3], [5, 6], None),
                // general
                ("Allen::starts", [2, 5], [2, 3], Some([2, 3])),
                ("Allen::finishes", [2, 5], [3, 5], Some([3, 5])),
                ("Allen::During", [2, 5], [3, 4], Some([3, 4])),
                ("Allen::equals", [2, 5], [2, 5], Some([2, 5])),
                ("Allen::overlaps_xy", [2, 5], [4, 7], Some([4, 5])),
                ("Allen::meets_xy", [2, 5], [6, 7], None),
                ("Allen::before_xy", [2, 5], [7, 7], None),
                ("Allen::overlaps_yx", [2, 5], [0, 3], Some([2, 3])),
                ("Allen::meets_yx", [2, 5], [0, 1], None),
                ("Allen::before_yx", [2, 5], [0, 0], None),
            ];
            for (code, x, y, expected) in periods.iter() {
                let x = ::Period::new(::Day::new(x[0]), ::Day::new(x[1] - x[0] + 1)).unwrap();
                let y = ::Period::new(::Day::new(y[0]), ::Day::new(y[1] - y[0] + 1)).unwrap();
                let expected = expected
                    .map(|x| ::Period::new(::Day::new(x[0]), ::Day::new(x[1] - x[0] + 1)).unwrap());
                if x.intersection(&y) != expected {
                    return Err(String::from(*code));
                }
                if x.intersect(&y) == expected.is_some() {
                    return Err(String::from(*code));
                }
            }
            Ok(())
        }
    }
}
