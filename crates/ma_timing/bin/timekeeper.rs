use clap::Parser;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::time::Duration;
use ma_timing::TimeKeeper;

use std::io::stdout;
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about= None)]
pub struct Configuration {
    #[arg(long, default_value_t = 10_000)]
    samples_per_datapoint: usize,
    #[arg(long, default_value_t = 256)]
    n_datapoints: usize,

    /// in secs
    #[arg(long, default_value_t = 0.5)]
    report_interval: f32,

    #[arg(long, default_value_t = 50)]
    minimum_duration: u64,
}

pub fn setup_logging(log_file: Option<&str>) {
    let mut t = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{:?}] ({}): {}",
                chrono::Local::now().format("[%Y-%m-%dT%H:%M:%S]"),
                std::thread::current().id(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());
    if let Some(log_file) = log_file {
        t = t.chain(fern::log_file(log_file).unwrap());
    }
    t.apply().unwrap();
}
fn main() {
    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();

    setup_logging(None);

    let config = Configuration::parse();
    let mut tc = TimeKeeper::new(
        0,
        Duration::from_secs_f32(config.report_interval),
        config.samples_per_datapoint,
        config.n_datapoints,
        ma_time::Nanos(config.minimum_duration)
    );
    tc.execute();
    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
