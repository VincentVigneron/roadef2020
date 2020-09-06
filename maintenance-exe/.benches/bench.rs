#![feature(test)]

extern crate maintenance;
extern crate maintenance_json;

const INPUT = "/home/vincent/Challenge/challenge-roadef-2020/A_set/A_05.json";

#[bench]
fn universe_ticks(b: &mut test::Bencher) {
    let instance = {
        let contents = maintenance_json::read_contents(&INPUT);
        let m = maintenance_json::read_json(&contents);
        maintenance_json::load_instance(m.unwrap()).unwrap()
    };
    let mut ls = search::LocalSearch::new(&instance.maintenance);
    ls.init();
}
