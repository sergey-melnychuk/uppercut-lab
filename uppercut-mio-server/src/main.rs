use std::error::Error;

use uppercut::api::Envelope;
use uppercut::core::System;
use uppercut::config::{Config, SchedulerConfig};
use uppercut::pool::ThreadPool;

mod protocol;
mod server;

use crate::server::{Listener, Start};

const MAX_CORES: usize = 4;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cores = std::cmp::min(MAX_CORES, num_cpus::get());

    let cfg = Config::new(
        SchedulerConfig::with_total_threads(cores + 2),
        Default::default());
    let pool = ThreadPool::for_config(&cfg);
    let sys = System::new("server", "localhost", &cfg);
    let run = sys.run(&pool).unwrap();

    let listener = Listener::listen("0.0.0.0:9000")?;
    run.spawn("server", move || Box::new(listener));
    run.send("server", Envelope::of(Start));

    std::thread::park();
    Ok(())
}
