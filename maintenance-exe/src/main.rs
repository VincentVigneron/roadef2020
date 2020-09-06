extern crate maintenance;
extern crate maintenance_json;

use std::env;
use std::fmt;
use std::io::prelude::*;
use std::time::Instant;

use maintenance::*;

struct ExportPlanning<'a> {
    planning: Planning,
    mapping: &'a MaintenanceMapping,
}

impl<'a> fmt::Display for ExportPlanning<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (iid, day) in self.planning.interventions.iter() {
            let intervention_name = &self.mapping.interventions[iid];
            let day = *day + Day::new(1);
            writeln!(f, "{} {}", intervention_name, day.get())?;
        }
        write!(f, "")
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let now = Instant::now();
    let m = {
        let contents = maintenance_json::read_contents(&args[1]);
        println!("Contents reading in: {}s", now.elapsed().as_secs());
        let m = maintenance_json::read_json(&contents);
        println!("Json reading in: {}s", now.elapsed().as_secs());
        maintenance_json::load_instance(m.unwrap())
    };
    match m {
        Some(instance) => {
            println!("loading: {}", instance.maintenance.ninterventions());
            println!("Conversion in: {}s", now.elapsed().as_secs());
            let mut ls = search::LocalSearch::new(&instance.maintenance);
            ls.init();
            let planning = ls.current_planning();
            let out = &args[2];
            let mut out = std::fs::File::create(out).expect("ok");
            let _ = write!(
                out,
                "{}",
                ExportPlanning {
                    planning,
                    mapping: &instance.mapping
                }
            );
            //write(&instance.mapping, &planning);
            println!("All in: {}s", now.elapsed().as_secs());
        }
        _ => println!("ERROR"),
    }
}
