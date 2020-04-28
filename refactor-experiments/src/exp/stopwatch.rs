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
    pub fn add(&mut self, s: String) {
        self.xs.push((Instant::now(), s))
    }
    pub fn report(&self) -> Option<String> {
        let total = self.xs.last()?.0 - self.xs.first()?.0;
        Some(format!("name: {}, duration: {}", &self.xs.first()?.1, total.as_millis()))
    }
}
