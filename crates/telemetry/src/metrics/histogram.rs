//! A bounded histogram of sample values.

use std::sync::Mutex;

#[derive(Debug)]
pub struct Histogram {
    pub bounds: Vec<f64>,
    counts: Mutex<Vec<u64>>,
    sum: Mutex<f64>,
}

impl Histogram {
    pub fn new(bounds: Vec<f64>) -> Self {
        let counts = vec![0; bounds.len()];
        Self {
            bounds,
            counts: Mutex::new(counts),
            sum: Mutex::new(0.0),
        }
    }

    pub fn latency() -> Self {
        Self::new(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0])
    }

    pub fn observe(&self, value: f64) {
        let mut c = self.counts.lock().unwrap();
        *self.sum.lock().unwrap() += value;
        for (i, bound) in self.bounds.iter().enumerate() {
            if value <= *bound {
                c[i] += 1;
                return;
            }
        }
    }

    pub fn count(&self) -> u64 {
        self.counts.lock().unwrap().iter().sum()
    }

    pub fn sum(&self) -> f64 {
        *self.sum.lock().unwrap()
    }
}