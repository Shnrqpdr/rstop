use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct History {
    capacity: usize,
    buf: VecDeque<f64>,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buf: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, v: f64) {
        if self.buf.len() == self.capacity {
            self.buf.pop_front();
        }
        self.buf.push_back(v);
    }

    pub fn as_u64_vec(&self) -> Vec<u64> {
        self.buf.iter().map(|v| v.round().max(0.0) as u64).collect()
    }

    pub fn as_slice_vec(&self) -> Vec<f64> {
        self.buf.iter().copied().collect()
    }

    pub fn last(&self) -> Option<f64> {
        self.buf.back().copied()
    }
}
