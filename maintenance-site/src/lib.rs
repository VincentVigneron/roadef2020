mod utils;
extern crate maintenance;
extern crate maintenance_capnproto;
extern crate maintenance_json;
extern crate serde_json;
extern crate uuid;
extern crate web_sys;

use std::time::Instant;

use maintenance::io::reader;
use maintenance::io::reader::*;
use wasm_bindgen::prelude::*;

use web_sys::console;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!($( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    type Buffer;
}
#[wasm_bindgen]
pub struct MaintenanceSummary {
    summary: maintenance_capnproto::MaintenanceSummaryReader,
}

#[wasm_bindgen]
impl MaintenanceSummary {
    pub fn from_bytes(data: &[u8]) -> Self {
        let summary = maintenance_capnproto::MaintenanceSummaryReader::from_bytes(data);
        match summary {
            Ok(summary) => MaintenanceSummary { summary: summary },
            _ => unimplemented!(),
        }
        //MaintenanceSummary {
        //summary: maintenance_capnproto::MaintenanceSummaryReader::from_bytes(data).expect("Ok"),
        //}
    }

    pub fn ndays(&self) -> u32 {
        self.summary.reader().ndays()
    }

    pub fn ninterventions(&self) -> u32 {
        self.summary.reader().ninterventions()
    }

    pub fn nresources(&self) -> u32 {
        self.summary.reader().nresources()
    }

    pub fn nscenarios(&self) -> u32 {
        self.summary.reader().nscenarios()
    }
}

//#[wasm_bindgen]
//pub struct MaintenanceSummary {
//summary: maintenance_json::MaintenanceSummary,
//}

//#[wasm_bindgen]
//impl MaintenanceSummary {
//pub fn from_utf8(data: &[u8]) -> Self {
//let contents = std::str::from_utf8(data).expect("Found invalid UTF-8");
//MaintenanceSummary {
//summary: serde_json::from_str(&contents).unwrap(),
//}
//}
//pub fn from_str(contents: &str) -> Self {
//MaintenanceSummary {
//summary: serde_json::from_str(&contents).unwrap(),
//}
//}

//pub fn ndays(&self) -> u32 {
//self.summary.ndays()
//}

//pub fn ninterventions(&self) -> u32 {
//self.summary.ninterventions()
//}

//pub fn nresources(&self) -> u32 {
//self.summary.nresources()
//}

//pub fn nscenarios(&self) -> u32 {
//self.summary.nscenarios()
//}
//}

#[wasm_bindgen]
pub struct Optim {
    maintenance: Option<MaintenanceOptimization>,
}

#[wasm_bindgen]
impl Optim {
    pub fn new() -> Optim {
        Optim { maintenance: None }
    }

    pub fn is_loaded(&self) -> bool {
        self.maintenance.is_some()
    }

    // use status enum
    pub fn load_from_bytes(&mut self, data: &[u8]) -> bool {
        let contents = std::str::from_utf8(data).expect("Found invalid UTF-8");
        let m = reader::read_json(contents);
        let m = reader::load_instance(m.unwrap());
        self.maintenance = Some(m.unwrap());
        self.is_loaded()
    }

    pub fn ninterventions(&self) -> u32 {
        self.maintenance
            .as_ref()
            .unwrap()
            .maintenance
            .ninterventions() as u32
    }

    pub fn ndays(&self) -> u32 {
        self.maintenance.as_ref().unwrap().maintenance.ndays() as u32
    }

    // use status enum
    pub fn load(&mut self, path: &str) -> bool {
        alert(&format!("load: {}", path));
        let contents = reader::read_contents(path);
        let now = Instant::now();
        println!("Contents reading in: {}s", now.elapsed().as_secs());
        let m = reader::read_json(&contents);
        println!("Json reading in: {}s", now.elapsed().as_secs());
        let m = reader::load_instance(m.unwrap());
        self.maintenance = Some(m.unwrap());
        self.is_loaded()
    }
}

#[wasm_bindgen]
pub struct Uuid {
    uuid: uuid::Uuid,
}
#[wasm_bindgen]

impl Uuid {
    pub fn from_bytes(data: &[u8]) -> Self {
        let uuid = maintenance_capnproto::capnp_uuid::decode(data).expect("OK");
        Uuid { uuid: uuid }
    }
}
