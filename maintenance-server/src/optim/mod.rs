use futures::future::{BoxFuture, FutureExt};
use futures::task::{waker_ref, ArcWake};
use rocket::http::ContentType;
use rocket::response::Response;
use rocket::Data;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::future::Future;
use std::sync::mpsc;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::Arc;
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::time::Duration;
use std::time::Instant;
use uuid::Uuid;

use maintenance::io::reader;

const SMALL_SIZE_LIMIT: u64 = 10 * 1024 * 1024;
const INSTANCE_SIZE_LIMIT: u64 = 750 * 1024 * 1024;
const INSTANCE_READING_BUFFER: usize = 100;

#[post("/optim/new", data = "<data>")]
pub fn receive_optim<'a, 'r>(
    content_type: &'a ContentType,
    data: Data,
    state: rocket::State<'r, ReadingSpawner>,
) -> std::result::Result<rocket::Response<'a>, rocket::http::Status> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file").size_limit(INSTANCE_SIZE_LIMIT),
    ]);
    let multipart_form_data = MultipartFormData::parse(content_type, data, options);
    let multipart_form_data = multipart_form_data.unwrap();
    let file = multipart_form_data.files.get("file");
    println!("opt: {:?}", file);
    match file {
        Some(file) => {
            // NOTE: take ownership of MultiPartFormData baceuse it is responsible
            // of tmp file deleting.
            let uuid = state.spawn(ReadingTask::new(multipart_form_data));
            let data = maintenance_capnproto::capnp_uuid::encode(&uuid).expect("OK");
            let response = Response::build()
                .status(rocket::http::Status::Accepted)
                .header(ContentType::Binary)
                .sized_body(std::io::Cursor::new(data))
                .finalize();
            return Ok(response);
        }
        None => println!("No file"),
    }
    unimplemented!()
}

pub fn new_async_reader() -> (ReadingExecutor, ReadingSpawner) {
    let (small_task_sender, small_ready_queue) = sync_channel(INSTANCE_READING_BUFFER);
    let (large_task_sender, large_ready_queue) = sync_channel(INSTANCE_READING_BUFFER);
    (
        ReadingExecutor {
            small_ready_queue,
            large_ready_queue,
        },
        ReadingSpawner {
            small_task_sender,
            large_task_sender,
        },
    )
}

#[derive(Clone)]
pub struct ReadingSpawner {
    small_task_sender: SyncSender<(Uuid, MultipartFormData)>,
    large_task_sender: SyncSender<(Uuid, MultipartFormData)>,
}

enum ReadingTask {
    Small(MultipartFormData),
    Large(MultipartFormData),
}

impl ReadingTask {
    fn new(data: MultipartFormData) -> Self {
        let file = data.files.get("file").unwrap();
        let file = std::fs::metadata(&file[0].path).expect("Ok");
        let size = file.len();
        if size > SMALL_SIZE_LIMIT {
            ReadingTask::Large(data)
        } else {
            ReadingTask::Small(data)
        }
    }
}

impl ReadingSpawner {
    // TODO(vincent): result based on file[0] existence
    fn spawn(&self, task: ReadingTask) -> Uuid {
        let uuid = Uuid::new_v4();
        match task {
            ReadingTask::Small(task) => self
                .small_task_sender
                .send((uuid, task))
                .expect("too many tasks queued"),
            ReadingTask::Large(task) => self
                .large_task_sender
                .send((uuid, task))
                .expect("too many tasks queued"),
        }
        uuid
    }
}

pub struct RunningExecutor {
    small_run_queue: Arc<Queue<Uuid>>,
    large_run_queue: Arc<Queue<Uuid>>,
    // 2threadss: if large empty then run small
}

pub struct ReadingExecutor {
    small_ready_queue: Receiver<(Uuid, MultipartFormData)>,
    large_ready_queue: Receiver<(Uuid, MultipartFormData)>,
}

pub struct ReadingExecutorJoinHandle {
    small_join_handle: std::thread::JoinHandle<()>,
    large_join_handle: std::thread::JoinHandle<()>,
}

impl ReadingExecutor {
    pub fn run(self) -> ReadingExecutorJoinHandle {
        let (small_ready_queue, large_ready_queue) =
            (self.small_ready_queue, self.large_ready_queue);
        ReadingExecutorJoinHandle {
            small_join_handle: std::thread::spawn(move || {
                loop {
                    //small_ready_queue: Receiver<Arc<ReadingTask>>,
                    //large_ready_queue: Receiver<Arc<ReadingTask>>,
                    let (uuid, data) = small_ready_queue.recv().unwrap();
                    let file = data.files.get("file").unwrap();
                    let path = &file[0].path;
                    let contents = reader::read_contents(path);
                    //let contents = reader::read_contents(&*path);
                    println!("Small handle: {:?}", file);
                    let now = Instant::now();
                    println!("Contents reading in: {}s", now.elapsed().as_secs());
                    let m = reader::read_json(&contents);
                    println!("Json reading in: {}s", now.elapsed().as_secs());
                    let m = reader::load_instance(m.unwrap());
                    match m {
                        Some(instance) => println!("OK"),
                        //optim.write().expect("ok").insert(uuid, instance);
                        _ => println!("ERROR"),
                    }
                }
            }),
            large_join_handle: std::thread::spawn(move || {
                loop {
                    //small_ready_queue: Receiver<Arc<ReadingTask>>,
                    //large_ready_queue: Receiver<Arc<ReadingTask>>,
                    let (uuid, data) = large_ready_queue.recv().unwrap();
                    let file = data.files.get("file").unwrap();
                    let path = &file[0].path;
                    let m = {
                        let contents = reader::read_contents(path);
                        //let contents = reader::read_contents(&*path);
                        println!("Large handle: {:?}", file);
                        let now = Instant::now();
                        println!("Contents reading in: {}s", now.elapsed().as_secs());
                        let m = reader::read_json(&contents);
                        println!("Json reading in: {}s", now.elapsed().as_secs());
                        reader::load_instance(m.unwrap())
                    }; // NOTE: drop contents here
                    match m {
                        Some(instance) => println!("OK"),
                        //optim.write().expect("ok").insert(uuid, instance);
                        _ => println!("ERROR"),
                    }
                }
            }),
        }
    }
}
