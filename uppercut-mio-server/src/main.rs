use std::error::Error;

use uppercut::api::Envelope;
use uppercut::core::System;
use uppercut::config::Config;
use uppercut::pool::ThreadPool;

mod protocol;
mod server;
use crate::server::{Listener, Start};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cfg = Config::default();
    let pool = ThreadPool::for_config(&cfg);
    let sys = System::new(cfg);
    let run = sys.run(&pool).unwrap();

    let listener = Listener::listen("0.0.0.0:9000")?;
    run.spawn("server", move || Box::new(listener));
    run.send("server", Envelope::of(Start, ""));

    std::thread::park();
    Ok(())
}
