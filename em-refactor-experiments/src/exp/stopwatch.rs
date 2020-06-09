use std::time::{Duration, Instant};

pub struct Stopwatch {
    xs: Vec<(Instant, String)>
}
impl Stopwatch {
    pub(crate) fn start(s: String) -> Self {
        Self {
            xs: vec![(Instant::now(), s)]
        }
    }
    pub(crate) fn add(&mut self, s: &str) {
        self.xs.push((Instant::now(), s.to_string()));
    }
    pub(crate) fn report(&self) -> Option<Vec<MetricsData>> {
        let mut ret = vec![];
        for i in 1..self.xs.len() {
            let diff = self.xs[i].0 - self.xs[i - 1].0;
            ret.push(MetricsData::new(diff, &self.xs[i].1));
        }
        let total = self.xs.last()?.0 - self.xs.first()?.0;
        ret.push(MetricsData::new(total, "total"));
        Some(ret)
    }
}

#[derive(Debug)]
pub struct MetricsData {
    pub duration: Duration,
    pub name: String
}
impl MetricsData {
    pub(crate) fn new(duration: Duration, name: &str) -> Self {
        Self {
            duration,
            name: name.to_string()
        }
    }
}