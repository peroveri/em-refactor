use std::time::Instant;

pub struct Stopwatch {
    xs: Vec<(Instant, String)>
}
impl Stopwatch {
    pub fn start(s: String) -> Self {
        Self {
            xs: vec![(Instant::now(), s)]
        }
    }
    pub fn add(&mut self, s: &str) {
        self.xs.push((Instant::now(), s.to_string()))
    }
    pub fn report(&self) -> Option<String> {
        let total = self.xs.last()?.0 - self.xs.first()?.0;
        let mut res = vec![];

        for i in 1..self.xs.len() {
            let diff = self.xs[i].0 - self.xs[i - 1].0;
            res.push(format!("name: {}, duration: {}", self.xs[i].1, diff.as_millis()));
        }
        Some(format!("name: {}, total duration: {}, parts: [{}]", &self.xs.first()?.1, total.as_millis(), res.join(",")))
    }
}
