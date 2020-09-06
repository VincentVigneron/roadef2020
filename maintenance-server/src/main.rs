#![feature(proc_macro_hygiene, decl_macro)]
#![feature(async_closure)]

#[macro_use]
extern crate rocket;
extern crate maintenance;
extern crate maintenance_capnproto;
extern crate maintenance_json;
extern crate rocket_cors;
extern crate rocket_multipart_form_data;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use maintenance::io::reader;
use maintenance_json::*;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;
use uuid::Uuid;

use rocket::config::{Config, Environment};
//use rocket::http::hyper::AllowedOrigins;
use rocket::config::Limits;
use rocket::http::ContentType;
use rocket::http::Method;
use rocket::request::Form;
use rocket::request::FormError;
use rocket::response;
use rocket::response::content;
use rocket::response::status;
use rocket::response::Response; // 1.
use rocket::Data;
use rocket_cors::{
    AllowedHeaders,
    AllowedOrigins,
    Cors,
    CorsOptions, // 3.
    Error,       // 2.
};
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, Repetition,
};

use std::path::{Path, PathBuf};

pub mod optim;

#[get("/test")]
fn index() -> status::Accepted<content::Json<&'static str>> {
    let content = "{\"value\": 0}";
    status::Accepted(Some(response::content::Json(content)))
}

#[post("/optim", data = "<data>")]
fn optim<'a, 'r>(
    content_type: &'a ContentType,
    data: Data,
    state: rocket::State<'r, mpsc::SyncSender<(Uuid, std::path::PathBuf)>>,
) -> std::result::Result<rocket::Response<'a>, rocket::http::Status> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file").size_limit(750 * 1024 * 1024),
    ]);
    let multipart_form_data = MultipartFormData::parse(content_type, data, options);
    let multipart_form_data = multipart_form_data.unwrap();
    let file = multipart_form_data.files.get("file");
    match file {
        Some(file) => {
            let uuid = Uuid::new_v4();
            let path = &file[0].path;
            // clone uuid we need to send it back to client
            // reemove clone when contents will be move to a thread
            let r = state.send((uuid, path.clone()));

            let contents = reader::read_contents(path);
            let now = Instant::now();
            println!("Contents reading in: {}s", now.elapsed().as_secs());
            let m = reader::read_json(&contents);
            println!("Json reading in: {}s", now.elapsed().as_secs());
            let m = reader::load_instance(m.unwrap());
            match m {
                Some(instance) => {
                    let test = maintenance_capnproto::MaintenanceSummaryBuilder::from_maintenance(
                        &instance.maintenance,
                    );
                    let data = test.bytes().expect("ok");
                    let response = Response::build()
                        .status(rocket::http::Status::Accepted)
                        .header(ContentType::Binary)
                        .sized_body(std::io::Cursor::new(data))
                        .finalize();
                    return Ok(response);
                    //status::Accepted(Some(response::content::Json(content)))
                }
                _ => println!("ERROR"),
            }
        }
        None => println!("No file"),
    }
    unimplemented!()
    //let content = format!("\"code\": {}", "error");
    //status::Accepted(Some(response::content::Json(content)))
}

#[post("/optim-json", data = "<data>")]
fn optim_json(content_type: &ContentType, data: Data) -> status::Accepted<content::Json<String>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file").size_limit(750 * 1024 * 1024),
    ]);
    let multipart_form_data = MultipartFormData::parse(content_type, data, options);
    let mut multipart_form_data = multipart_form_data.unwrap();
    let file = multipart_form_data.files.remove("file");
    match file {
        Some(file) => {
            let path = &file[0].path;

            let contents = reader::read_contents(path);
            let now = Instant::now();
            println!("Contents reading in: {}s", now.elapsed().as_secs());
            let m = reader::read_json(&contents);
            println!("Json reading in: {}s", now.elapsed().as_secs());
            let m = reader::load_instance(m.unwrap());
            match m {
                Some(instance) => {
                    let response = maintenance_json::MaintenanceSummary::from_maintenance(
                        &instance.maintenance,
                    );
                    return status::Accepted(Some(response::content::Json(
                        serde_json::to_string(&response).unwrap(),
                    )));
                }
                _ => println!("ERROR"),
            }
        }
        None => println!("No file"),
    }
    let content = format!("\"code\": {}", "error");
    status::Accepted(Some(response::content::Json(content)))
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:8080",
        "http://127.0.0.1:8080",
        "http://0.0.0.0:8080",
        "http://localhost:8000",
        "http://127.0.0.1:8000",
        "http://0.0.0.0:8000",
        "http://192.168.56.3:8000",
        "http://192.168.56.3:8080",
    ]);

    CorsOptions {
        // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(), // 1.
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Content-type",
            "Access-Control-Allow-Origin", // 6.
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

// TODO(vincent); use Rocket.toml
fn rocket() -> rocket::Rocket {
    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8000)
        .finalize()
        .expect("K");
    rocket::custom(config)
        .mount("/", routes![index, optim, optim_json, optim::receive_optim])
        .attach(make_cors())
}

enum OptimizationRequest {
    Status(Uuid),
    Planning(Uuid),
}

fn main() {
    let optim: HashMap<Uuid, maintenance::io::reader::MaintenanceOptimization> = HashMap::new();
    let optim = Arc::new(RwLock::new(optim));
    // load
    //let (instance_tx, instance_rx) =
    //sync_channel::<(Uuid, MultipartFormData)>(INSTANCE_READING_BUFFER);
    ////sync_channel::<(Uuid, std::path::PathBuf)>(INSTANCE_READING_BUFFER);
    //{
    //let optim = Arc::clone(&optim);
    //thread::spawn(move || loop {
    ////let (uuid, path) = instance_rx.recv().unwrap();
    //let (uuid, data) = instance_rx.recv().unwrap();

    //let file = data.files.get("file").unwrap();
    //let path = &file[0].path;
    //println!("recv: ({:?}, {:?})", uuid, uuid);
    //let contents = reader::read_contents(path);
    ////let contents = reader::read_contents(&*path);
    //println!("file exits");
    //let now = Instant::now();
    //println!("Contents reading in: {}s", now.elapsed().as_secs());
    //let m = reader::read_json(&contents);
    //println!("Json reading in: {}s", now.elapsed().as_secs());
    //let m = reader::load_instance(m.unwrap());
    //match m {
    //Some(instance) => {
    //optim.write().expect("ok").insert(uuid, instance);
    //}
    //_ => println!("ERROR"),
    //}
    //});
    //}
    //let (optim_tx, optim_rx) = channel::<OptimizationRequest>();
    let (executor, spawner) = optim::new_async_reader();
    let _executor = executor.run();
    rocket()
        //.manage(instance_tx)
        .manage(spawner.clone())
        .launch();
}
