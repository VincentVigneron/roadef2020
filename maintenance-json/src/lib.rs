extern crate maintenance;
extern crate serde;
extern crate serde_json;

use maintenance::*;
use serde::Deserialize;
use std::collections::btree_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::ops::{Add, Sub};
use std::path::Path;
use std::time::Duration;

use serde::de::{Deserializer, Unexpected, Visitor};

use std::collections::BTreeMap as OrderedMap;
use std::collections::HashMap as Map;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
/// Day identifier
pub struct SerdeDay(usize);

impl SerdeDay {
    pub fn new(id: usize) -> Self {
        SerdeDay(id)
    }

    pub fn get(&self) -> usize {
        let SerdeDay(id) = *self;
        id
    }
}

impl Sub for SerdeDay {
    type Output = SerdeDay;
    fn sub(self, other: SerdeDay) -> SerdeDay {
        let SerdeDay(me) = self;
        let SerdeDay(other) = other;
        SerdeDay(me - other)
    }
}
impl Add for SerdeDay {
    type Output = SerdeDay;
    fn add(self, other: SerdeDay) -> SerdeDay {
        let SerdeDay(me) = self;
        let SerdeDay(other) = other;
        SerdeDay(me + other)
    }
}

impl Into<Day> for SerdeDay {
    fn into(self) -> Day {
        Day::new(self.get())
    }
}

struct SerdeDayVisitor;
impl<'de> Visitor<'de> for SerdeDayVisitor {
    type Value = SerdeDay;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "f64, usize, str")
    }

    fn visit_f64<E>(self, v: f64) -> Result<SerdeDay, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeDay::new(v as usize))
    }

    fn visit_f32<E>(self, v: f32) -> Result<SerdeDay, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeDay::new(v as usize))
    }

    fn visit_str<E>(self, s: &str) -> Result<SerdeDay, E>
    where
        E: serde::de::Error,
    {
        s.parse::<usize>()
            .map(SerdeDay::new)
            .map_err(|_| E::invalid_value(Unexpected::Str(s), &"an unsigned int as a string."))
    }
}

impl<'de> Deserialize<'de> for SerdeDay {
    fn deserialize<D>(deserializer: D) -> Result<SerdeDay, D::Error>
    where
        D: Deserializer<'de>,
    {
        let day = deserializer.deserialize_any(SerdeDayVisitor)?;
        // SerdeDays are indexed from 1
        Ok(SerdeDay(day.get() - 1usize))
    }
}

#[derive(Deserialize)]
pub struct SerdeIntervention<'a> {
    #[serde(rename(deserialize = "tmax"))]
    last_possible_start: SerdeDay, //  last _posiblt day
    #[serde(rename(deserialize = "Delta"))]
    delta: Vec<SerdeDay>, // duration in days
    #[serde(borrow)]
    workload: Map<&'a str, Map<SerdeDay, Map<SerdeDay, f64>>>,
    risk: Map<SerdeDay, Map<SerdeDay, Vec<f64>>>,
}

#[derive(Deserialize)]
struct SerdeResource {
    min: Vec<f64>,
    max: Vec<f64>,
}

#[derive(Deserialize)]
pub struct SerdeMaintenance<'a> {
    #[serde(rename(deserialize = "Resources"))]
    resources: Map<&'a str, SerdeResource>,
    #[serde(rename(deserialize = "Seasons"))]
    seasons: OrderedMap<&'a str, Vec<SerdeDay>>,
    #[serde(rename(deserialize = "Interventions"))]
    #[serde(borrow)]
    interventions: OrderedMap<&'a str, SerdeIntervention<'a>>,
    #[serde(rename(deserialize = "Exclusions"))]
    #[serde(borrow)]
    exclusions: Map<&'a str, Vec<&'a str>>,
    #[serde(rename(deserialize = "T"))]
    ndays: usize,
    #[serde(rename(deserialize = "Scenarios_number"))]
    scenarios_number: Vec<usize>, // expected values
    #[serde(rename(deserialize = "Quantile"))]
    quantile: f64,
    #[serde(rename(deserialize = "Alpha"))]
    alpha: f64,
    #[serde(rename(deserialize = "ComputationTime"))]
    computation_time: u64,
}

struct WorkingInterventions {
    interventions: Box<[Intervention]>,
    interventions_codes: HashMap<String, IID>,
}

fn compute_interventions(
    json_interventions: OrderedMap<&str, SerdeIntervention>,
    resources_codes: &HashMap<String, RID>,
    seasons: &[SID],
    nseasons: usize,
    scenarios_number: usize,
    ndays: usize,
) -> WorkingInterventions {
    let mut interventions: Vec<Intervention> = Vec::new();
    interventions.reserve(json_interventions.len());
    let mut codes: HashMap<String, IID> = HashMap::new();
    for (iid, (name, intervention)) in json_interventions.into_iter().enumerate() {
        codes.insert(name.to_owned(), IID::new(iid));
        interventions.push(create_intervention(
            intervention,
            &resources_codes,
            &seasons,
            nseasons,
            scenarios_number,
            ndays,
        ));
    }
    WorkingInterventions {
        interventions: interventions.into_boxed_slice(),
        interventions_codes: codes,
    }
}

fn create_intervention(
    json_intervention: SerdeIntervention,
    resources_codes: &HashMap<String, RID>,
    seasons: &[SID],
    nseasons: usize,
    scenarios_number: usize,
    ndays: usize, //scenarios_number: &[usize],
) -> Intervention {
    let last_day = Day::new(ndays - 1);
    let periods = json_intervention
        .delta
        .into_iter()
        .enumerate()
        .map(|(start, duration)| {
            let start = SerdeDay::new(start).into();
            let duration = (duration + SerdeDay::new(1)).into();
            Period::new(start, duration).unwrap()
        })
        .take_while(|&p| p.end() <= last_day)
        .collect::<Box<[Period]>>();
    let last_possible_start = json_intervention.last_possible_start;
    let (resources, workloads) =
        create_workloads(json_intervention.workload, &periods, &resources_codes);
    let risks = create_risks(json_intervention.risk, &periods, scenarios_number);
    let seasons_of_periods = create_seasons_of_periods(&periods, &seasons, nseasons);

    maintenance::Intervention::builder()
        .set_latest_start(last_possible_start)
        .set_periods(periods)
        .set_seasons(seasons_of_periods)
        .set_risks(risks)
        .set_workloads(workloads)
        .set_resources(resources)
        .build()
}

fn create_risks(
    risks: Map<SerdeDay, Map<SerdeDay, Vec<f64>>>,
    periods: &[Period],
    scenarios_number: usize,
    //scenarios_number: &[usize],
) -> Risks {
    let period_sum = periods.iter().map(|p| p.duration().get()).sum::<usize>();

    //let cumuled_sc = scenarios_number
    //.iter()
    //.scan(0usize, |state, n| {
    //*state = *state + n;
    //Some(*state)
    //})
    //.collect::<Box<[_]>>();
    let period_slice = std::iter::once(0usize)
        .chain(periods.iter().scan(0usize, |state, p| {
            // NOTE(vincent) check if end or end +1
            let nb_scenarios = scenarios_number; //cumuled_sc[p.end().get()] - cumuled_sc[p.start().get()];
            *state += p.duration().get() * nb_scenarios;
            Some(*state)
        }))
        .collect::<Box<[_]>>();
    let mut final_risks = vec![0.0f64; period_sum * scenarios_number].into_boxed_slice();
    //let mut final_risks = vec![0.0f64; period_slice[period_slice.len() - 1]].into_boxed_slice();
    for (current_day, risks) in risks.into_iter() {
        for (starting_day, risks) in risks.into_iter() {
            // NOTE(Vincent): No OOB because period is indexed by starting days and
            // period_slice len is period len + 1.
            let idx = starting_day.get();
            let begin = period_slice[idx];
            let end = period_slice[idx + 1];
            let risk_slice = &mut final_risks[begin..end];
            let idx = current_day.get() - starting_day.get();
            let begin = idx * scenarios_number;
            let end = (idx + 1) * scenarios_number;
            let risk_slice = &mut risk_slice[begin..end];
            for (spos, risk) in risks.into_iter().enumerate() {
                risk_slice[spos] = risk;
            }
        }
    }
    Risks::builder()
        .set_nscenarios(scenarios_number)
        .set_periods(period_slice)
        .set_risks(final_risks)
        .build()
}

fn create_workloads(
    serde_workloads: Map<&str, Map<SerdeDay, Map<SerdeDay, f64>>>,
    periods: &[Period],
    resources_codes: &HashMap<String, RID>,
) -> (Box<[RID]>, Box<[Workload]>) {
    let nresources = serde_workloads.len();
    let mut final_workloads = periods
        .iter()
        .map(|p| p.duration())
        .flat_map(|dur| {
            (0..nresources).map(move |rpos| (rpos, vec![0.0f64; dur.get()].into_boxed_slice()))
        })
        .collect::<Vec<(usize, Box<[f64]>)>>();
    let mut resources: Vec<RID> = Vec::new();

    for (rpos, (rcode, workloads)) in serde_workloads.into_iter().enumerate() {
        resources.push(resources_codes[rcode]);
        for (current_day, workloads) in workloads.into_iter() {
            for (starting_day, workload) in workloads.into_iter() {
                final_workloads[rpos + starting_day.get() * nresources].1
                    [current_day.get() - starting_day.get()] = workload;
            }
        }
    }
    let final_workloads = final_workloads
        .into_iter()
        .map(|(rpos, workloads)| Workload::new(resources[rpos], workloads))
        .collect();
    (resources.into_boxed_slice(), final_workloads)
}

fn create_seasons_of_period(period: &Period, seasons: &[SID], nseasons: usize) -> Seasons {
    //let mut seasons_of_p = BitSet::from_bit_vec(BitVec::from_elem(nseasons, false));
    // add one free season
    let mut seasons_of_p = Seasons::with_capacity(nseasons);
    let days = period.days_exclusive();
    for day in days.0.get()..days.1.get() {
        if day < seasons.len() {
            let sid = seasons[day].get();
            seasons_of_p.set(sid, true);
        }
    }
    seasons_of_p
}

fn create_seasons_of_periods(
    periods: &[Period],
    seasons: &[SID],
    nseasons: usize,
) -> Box<[Seasons]> {
    periods
        .iter()
        .map(|period| create_seasons_of_period(period, seasons, nseasons))
        .collect()
}

#[derive(Debug)]
pub enum SerdeMaintenanceError {
    IO(std::io::Error),
    JSON(serde_json::Error),
}

pub fn read_contents<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => contents,
        Err(_err) => unimplemented!(),
        //Err(SerdeMaintenanceError::IO(err)),
    }
}

pub fn read_json(
    contents: &str,
) -> std::result::Result<SerdeMaintenance<'_>, SerdeMaintenanceError> {
    let scenario = serde_json::from_str(contents);
    match scenario {
        Ok(scenario) => Ok(scenario),
        Err(err) => Err(SerdeMaintenanceError::JSON(err)),
    }
}

pub enum InstanceError {
    IO(std::io::Error),
    JSON(serde_json::Error),
}

pub struct WorkingSeasons<'a> {
    seasons_codes: HashMap<&'a str, SID>,
    nseasons: usize,
    season_of_days: Box<[SID]>,
}

fn list_of_seasons<'a, 'b: 'a>(
    seasons: &'a OrderedMap<&'b str, Vec<SerdeDay>>,
    ndays: SerdeDay,
) -> WorkingSeasons<'b> {
    let nseasons = seasons.len() + 1;
    let mut seasons_codes: HashMap<&'b str, SID> = HashMap::new();
    let mut season_of_days = vec![SID::new(nseasons - 1); ndays.get()].into_boxed_slice();
    for (sid, (season, days)) in seasons.iter().enumerate() {
        seasons_codes.insert(season, SID::new(sid));
        for day in days.iter() {
            season_of_days[day.get()] = SID::new(sid);
        }
    }
    WorkingSeasons {
        seasons_codes,
        nseasons,
        season_of_days,
    }
}

fn update_exclusion(
    exclusions: &mut Option<OrderedMap<IID, Seasons>>,
    iid: IID,
    sid: SID,
    nseasons: usize,
) {
    if let Some(ref mut exclusion) = *exclusions {
        match exclusion.entry(iid) {
            Entry::Occupied(seasons) => {
                seasons.into_mut().set(sid.get(), true);
            }
            Entry::Vacant(exclusion) => {
                let mut seasons = Seasons::with_capacity(nseasons);
                seasons.set(sid.get(), true);
                exclusion.insert(seasons);
            }
        }
    } else {
        let mut seasons = Seasons::with_capacity(nseasons);
        seasons.set(sid.get(), true);
        let mut new_exclusion = OrderedMap::new();
        new_exclusion.insert(iid, seasons);
        *exclusions = Some(new_exclusion);
    }
}

fn compute_exclusions(
    json_exclusions: Map<&str, Vec<&str>>,
    interventions_codes: &HashMap<String, IID>,
    ninterventions: usize,
    season_codes: &HashMap<&str, SID>,
    nseasons: usize,
) -> Box<[InterventionExclusions]> {
    let mut exclusions: Vec<Option<OrderedMap<IID, Seasons>>> = vec![None; ninterventions];

    for (_, excl) in json_exclusions {
        let i1 = interventions_codes[excl[0]];
        let i2 = interventions_codes[excl[1]];
        let season = season_codes[excl[2]];
        let (i1, i2) = if i1 > i2 { (i1, i2) } else { (i2, i1) };
        update_exclusion(&mut exclusions[i1.get()], i2, season, nseasons);
        update_exclusion(&mut exclusions[i2.get()], i1, season, nseasons);
    }
    exclusions
        .into_iter()
        .map(|excl| InterventionExclusions {
            exclusions: match excl {
                Some(excl) => excl.into_iter().collect::<Box<[(IID, Seasons)]>>(),
                None => vec![].into_boxed_slice(),
            },
        })
        .collect()
}

struct WorkingResources {
    resources: Box<[Resource]>,
    resources_codes: HashMap<String, RID>,
}

fn list_of_resources(json_resources: Map<&str, SerdeResource>) -> WorkingResources {
    let mut resources: Vec<Resource> = Vec::new();
    let mut resources_codes: HashMap<String, RID> = HashMap::new();
    for (idx, (rname, json_resource)) in json_resources.into_iter().enumerate() {
        resources.push(Resource {
            min: json_resource.min.into(),
            max: json_resource.max.into(),
        });
        resources_codes.insert(rname.to_owned(), RID::new(idx));
    }
    WorkingResources {
        resources: resources.into_boxed_slice(),
        resources_codes,
    }
}

pub fn load_instance(maintenance: SerdeMaintenance) -> Option<MaintenanceOptimization> {
    let nscenarios = *maintenance.scenarios_number.iter().max().unwrap();
    let quantile = maintenance.quantile;
    let alpha = maintenance.alpha;
    let computation_time = maintenance.computation_time;
    let ndays = SerdeDay::new(maintenance.ndays);
    let working_seasons = list_of_seasons(&maintenance.seasons, ndays);
    let working_resources = list_of_resources(maintenance.resources);
    let working_interventions = compute_interventions(
        maintenance.interventions,
        &working_resources.resources_codes,
        &working_seasons.season_of_days,
        working_seasons.nseasons,
        //&maintenance.scenarios_number,
        nscenarios,
        ndays.get(),
    );
    let exclusions = compute_exclusions(
        maintenance.exclusions,
        &working_interventions.interventions_codes,
        working_interventions.interventions.len(),
        &working_seasons.seasons_codes,
        working_seasons.nseasons,
    );
    let scenarios_number = maintenance.scenarios_number;
    Some(MaintenanceOptimization {
        maintenance: Maintenance::builder()
            .set_ndays(ndays.get())
            .set_quantile(quantile)
            .set_alpha(alpha)
            .set_interventions(working_interventions.interventions)
            .set_resources(working_resources.resources)
            .set_exclusions(exclusions)
            .set_scenarios_number(scenarios_number)
            .build(),
        mapping: MaintenanceMapping {
            resources: working_resources
                .resources_codes
                .into_iter()
                .map(|(code, rid)| (rid, code))
                .collect(),
            interventions: working_interventions
                .interventions_codes
                .into_iter()
                .map(|(code, rid)| (rid, code))
                .collect(),
            seasons: working_seasons
                .seasons_codes
                .into_iter()
                .map(|(code, rid)| (rid, code.to_owned()))
                .collect(),
        },
        computation_time: Duration::new(computation_time, 0),
    })
}
