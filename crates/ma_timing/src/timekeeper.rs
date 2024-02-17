use super::messages::LatencyMessage;
use ma_time::*;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ma_queues::{Consumer, ReadError};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph};
use ratatui::{prelude::*, Terminal};
use std::time::SystemTime;
use std::{fmt::Debug, io::stdout};

use crate::messages::{TimingMeasurement, LatencyMeasurement, TimingMessage};
use crate::utils::CircularBuffer;
//TODO: Have tuple of 2 timingdatas pls
/// Keep track of msg latencies
/// All in nanos
#[derive(Debug, Clone)]
pub struct TimingData {
    title: String,
    // These are updated when Start
    measurements: Vec<Nanos>,
    averages: CircularBuffer<Nanos>,

    min: Nanos,
    max: Nanos,
    median: Nanos,

    clock_overhead: Nanos,

    samples_per_datapoint: usize,
    n_messages: usize,
    last_report: Instant,
    minimum_duration: Nanos,
}

impl TimingData {
    pub fn new(
        title: String,
        samples_per_datapoint: usize,
        n_datapoints: usize,
        clock_overhead: Nanos,
        minimum_duration: Nanos,
    ) -> Self {
        let measurements = Vec::with_capacity(samples_per_datapoint);
        let averages = CircularBuffer::new(n_datapoints);

        Self {
            title,
            measurements,
            averages,
            min: Nanos(0),
            max: Nanos(0),
            median: Nanos(0),
            clock_overhead,
            samples_per_datapoint,
            n_messages: 0,
            last_report: Instant::now(),
            minimum_duration
        }
    }

    fn min(&self) -> (Nanos, usize) {
        let (mut m, mut minid) = (Nanos::MAX, 0);
        for (id, v) in self.averages.iter().enumerate() {
            if *v < m {
                m = *v;
                minid = id;
            }
        }
        (m, minid)
    }

    fn max(&self) -> (Nanos, usize) {
        let (mut m, mut maxid) = (Nanos::ZERO, 0);
        for (id, v) in self.averages.iter().enumerate() {
            if *v > m {
                m = *v;
                maxid = id;
            }
        }
        (m, maxid)
    }

    // Also sets last min last max
    fn cur_avg(&self) -> Nanos {
        let n = self.measurements.len();
        if n == 0 {
            Nanos::ZERO
        } else {
            self.corrected_or_zero(self.measurements.iter().sum::<Nanos>() / n)
        }
    }

    fn avg(&self) -> Nanos {
        let n =self.averages.len();
        if n == 0 {
            Nanos::ZERO
        } else {
            let tot: Nanos = self.averages.iter().sum();
            tot / n as u64
        }
    }

    fn corrected_or_zero(&self, t: Nanos) -> Nanos {
        if t > self.clock_overhead {
            t - self.clock_overhead
        } else {
            Nanos::ZERO
        }
    }

    fn register_datapoint(&mut self) {
        let n = self.measurements.len();
        if n == 0 {
            return;
        }
        self.measurements.sort();

        self.max = self.corrected_or_zero(*self.measurements.last().unwrap());
        self.min = self.corrected_or_zero(*self.measurements.first().unwrap());

        self.median = self.corrected_or_zero(self.measurements[n / 2]);
        self.averages.push(self.cur_avg());
        self.measurements.clear();
    }

    fn track(&mut self, el: Nanos) -> bool {
        if el < self.minimum_duration {
            return false
        }
        self.n_messages += 1;
        self.measurements.push(el);
        if self.measurements.len() == self.samples_per_datapoint {
            self.register_datapoint();
            true
        } else {
            false
        }
    }

    pub fn report(&mut self, name: &str, frame: &mut Frame, rect: Rect) {
        self.register_datapoint();

        let avg = self.averages.last();
        let text: Vec<Line> = vec![
            format!("{} Report for {name}", self.title).into(),
            format!(
                "msgs: {} ({} msg/ms)",
                self.n_messages,
                self.n_messages as f64 / self.last_report.elapsed().as_millis()
            )
            .into(),
            format!("avg: {avg}").into(),
            format!("median: {}", self.median).into(),
            format!("min: {}", self.min).into(),
            format!("max: {}", self.max).into(),
        ]
        .into();

        let sub_layout = Layout::new()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(text.len() as u16), Constraint::Min(20)])
            .split(rect);

        frame.render_widget(Paragraph::new(text), sub_layout[0]);
        let to_plot: Vec<(f64, f64)> = self
            .averages
            .iter()
            .enumerate()
            .map(|(i, &p)| (i as f64, p.0 as f64))
            .collect();

        let midpoint = self.avg();
        let min = self.averages.iter().min().unwrap();
        let max = self.averages.iter().max().unwrap();
        let ylabels = vec![
            format!("{min}").into(),
            format!("{midpoint}").into(),
            format!("{max}").into(),
        ];

        let xlabels = vec![
            format!("0").into(),
            format!("{}", self.averages.len()).into(),
        ];

        let xaxis = Axis::default()
            .bounds([0.0, to_plot.len() as f64])
            .style(Style::default().fg(Color::LightBlue))
            .labels(xlabels.into());
        let yaxis = Axis::default()
            .bounds([min.0 as f64, max.0 as f64])
            .style(Style::default().fg(Color::LightBlue))
            .labels(ylabels.into());
        let chart = Chart::new(vec![Dataset::default()
            .name(format!("{} averages", self.title))
            .data(&to_plot)])
        .x_axis(xaxis)
        .y_axis(yaxis);
        frame.render_widget(
            chart.block(Block::new().borders(Borders::ALL)),
            sub_layout[1],
        );
        self.n_messages = 0;
    }
}

struct TimerData {
    name: String,
    latency_data: TimingData,
    business_data: TimingData,
    direction: Direction,
}
impl TimerData {
    pub fn new(
        name: String,
        samples_per_datapoint: usize,
        n_datapoints: usize,
        clock_overhead: Nanos,
        minimum_duration: Nanos
    ) -> Self {
        Self {
            name,
            latency_data: TimingData::new(
                "Latency".into(),
                samples_per_datapoint,
                n_datapoints,
                clock_overhead,
                minimum_duration
            ),
            business_data: TimingData::new(
                "Business".into(),
                samples_per_datapoint,
                n_datapoints,
                clock_overhead,
                minimum_duration
            ),
            direction: Direction::Horizontal,
        }
    }

    pub fn report(&mut self, frame: &mut Frame, rect: Rect) {
        let layout = Layout::new()
            .direction(self.direction)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rect);
        self.latency_data.report(&self.name, frame, layout[0]);
        self.business_data.report(&self.name, frame, layout[1]);
    }

    pub fn track_latency(&mut self, msg: &LatencyMeasurement) -> bool {
        self.latency_data.track(msg.latency())
    }

    pub fn track_business(&mut self, msg: &TimingMeasurement) -> bool {
        self.business_data.track(msg.elapsed())
    }
}

fn black_box<T>(dummy: T) -> T {
    unsafe { std::ptr::read_volatile(&dummy) }
}

pub fn clock_overhead() -> Nanos {
    let start = Instant::now();
    for _ in 0..1_000_000 {
        black_box(Instant::now());
    }
    let end = Instant::now();
    (end - start) / 1_000_000 as u32
}

//TODO: Have a built in threshold to throw out timing messages that mean nothing
pub struct TimeKeeper {
    core: usize,
    report_interval: std::time::Duration,
    samples_per_datapoint: usize,
    n_datapoints: usize,
    minimum_duration: Nanos
}

impl TimeKeeper {
    pub fn new(
        core: usize,
        report_interval: std::time::Duration,
        samples_per_datapoint: usize,
        n_datapoints: usize,
        minimum_duration: Nanos,
    ) -> Self {
        Self {
            core,
            report_interval,
            samples_per_datapoint,
            n_datapoints,
            minimum_duration
        }
    }

    pub fn execute(&mut self) {
        core_affinity::set_for_current(core_affinity::CoreId { id: self.core });
        let clock_overhead = clock_overhead();

        // let mut names = Vec::new();
        let mut time_datas: Vec<TimerData> = Vec::new();
        let mut latency_consumers = Vec::new();
        let mut business_consumers = Vec::new();
        let rep_interval = self.report_interval;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        let mut curid = 0;
        let mut stacking_direction = Direction::Vertical;
        terminal.clear();

        loop {
            for entry in std::fs::read_dir(super::QUEUE_DIR)
                .unwrap()
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let name = entry.path().as_os_str().to_str().unwrap().to_string();
                if name.contains("latency") {
                    let (dir, real_name) = name.split_once("latency-").unwrap();

                    if time_datas.iter().find(|d| d.name == real_name).is_none() {
                        let d = TimerData::new(
                            real_name.to_string().clone(),
                            self.samples_per_datapoint,
                            self.n_datapoints,
                            clock_overhead,
                            self.minimum_duration
                        );
                        time_datas.push(d);
                        let latency_q = ma_queues::Queue::shared(format!("{}/latency-{real_name}", crate::QUEUE_DIR), crate::QUEUE_SIZE, ma_queues::QueueType::SPMC).expect("couldn't open latency queue");
                        latency_consumers.push(Consumer::from(latency_q));
                        let business_q =
                            ma_queues::Queue::shared(format!("{}/timing-{real_name}", crate::QUEUE_DIR), crate::QUEUE_SIZE, ma_queues::QueueType::SPMC).expect("couldn't open timing queue");
                        business_consumers.push(Consumer::from(business_q));
                    }
                }
            }
            for _ in 0..40 {
                std::thread::sleep(self.report_interval / 50);
                handle_latency_messages(&mut time_datas, &mut latency_consumers);
                handle_business_messages(&mut time_datas, &mut business_consumers);
                if event::poll(Duration::ZERO).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        if matches!(key.kind, KeyEventKind::Press) {
                            match key.code {
                                KeyCode::Char('q') => return,
                                KeyCode::Char('s') => {
                                    for d in &mut time_datas {
                                        d.direction = stacking_direction;
                                    }
                                    stacking_direction = match stacking_direction {
                                        Direction::Horizontal => Direction::Vertical,
                                        Direction::Vertical => Direction::Horizontal,
                                    };
                                    terminal.draw(|frame| {
                                        draw(frame, &mut time_datas, curid);
                                    });
                                }

                                KeyCode::Down => {
                                    curid += 1;
                                    if curid > time_datas.len() - 1 {
                                        curid = 0;
                                    }
                                    terminal.draw(|frame| {
                                        draw(frame, &mut time_datas, curid);
                                    });
                                }
                                KeyCode::Up => {
                                    if curid == 0 {
                                        curid = time_datas.len() - 1;
                                    } else {
                                        curid -= 1;
                                    }
                                    terminal.draw(|frame| {
                                        draw(frame, &mut time_datas, curid);
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // self.maybe_report(&mut time_datas, &mut terminal);
            }
            terminal.draw(|frame| {
                draw(frame, &mut time_datas, curid);
            });
        }
    }
}
fn handle_latency_messages<'a>(
    time_datas: &mut Vec<TimerData>,
    readers: &mut Vec<Consumer<'a, LatencyMeasurement>>,
) {
    let mut msg = Default::default();
    for (d, r) in time_datas.iter_mut().zip(readers) {
        let mut got_datapoint = false;
        while !got_datapoint {
            match r.try_consume(&mut msg) {
                Ok(()) => {
                    got_datapoint = d.track_latency(&msg);
                }
                Err(ReadError::Empty) => got_datapoint = true,
                Err(ReadError::SpedPast) => r.recover_after_error(),
            }
        }
    }
}
fn handle_business_messages<'a>(
    time_datas: &mut Vec<TimerData>,
    readers: &mut Vec<Consumer<'a, TimingMeasurement>>,
) {
    let mut msg = Default::default();
    for (d, r) in time_datas.iter_mut().zip(readers) {
        let mut got_datapoint = false;
        while !got_datapoint {
            match r.try_consume(&mut msg) {
                Ok(()) => {
                    got_datapoint = d.track_business(&msg);
                }
                Err(ReadError::Empty) => got_datapoint = true,
                Err(ReadError::SpedPast) => r.recover_after_error(),
            }
        }
    }
}

fn draw(frame: &mut Frame, time_datas: &mut Vec<TimerData>, curid: usize) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.size());

    let namelist: Text = Vec::from_iter(time_datas.iter().enumerate().map(|(i, d)| {
        if i == curid {
            Span::styled(d.name.clone(), Style::default().bg(Color::Gray)).into()
        } else {
            Span::raw(d.name.clone()).into()
        }
    }))
    .into();

    frame.render_widget(
        Paragraph::new(namelist).block(Block::new().title("Timers").borders(Borders::ALL)),
        layout[0],
    );

    time_datas[curid].report(frame, layout[1]);
}
