use std::collections::HashMap;

pub struct Counter {
    max: usize,
    n: usize,
}

impl Counter {
    pub fn new(max: usize) -> Self {
        Self { max, n: 0 }
    }

    pub fn v(&self) -> usize {
        self.n
    }

    pub fn vv(&self) -> Option<usize> {
        if self.n == 0 {
            None
        } else {
            Some(self.n)
        }
    }

    pub fn inc(&mut self) -> usize {
        self.n = (self.n + 1).rem_euclid(self.max);
        self.n
    }
    pub fn dec(&mut self) -> usize {
        self.n = (self.n - 1).rem_euclid(self.max);
        self.n
    }

    pub fn add(&mut self, n: usize) -> usize {
        self.n = (self.n + n).rem_euclid(self.max);
        self.n
    }
    pub fn sub(&mut self, n: usize) -> usize {
        self.n = (self.n - n).rem_euclid(self.max);
        self.n
    }
}

#[derive(Default)]
pub struct CounterEnv {
    map: HashMap<String, Counter>,
}

impl CounterEnv {
    pub fn with(mut self, key: &str, n: usize) -> Self {
        self.map.insert(key.to_owned(), Counter::new(n));
        self
    }

    pub fn get(&self, key: &str) -> &Counter {
        self.map
            .get(key)
            .unwrap_or_else(|| panic!("no such counter {}", key))
    }
    pub fn get_mut(&mut self, key: &str) -> &mut Counter {
        self.map
            .get_mut(key)
            .unwrap_or_else(|| panic!("no such counter {}", key))
    }

    pub fn v(&self, key: &str) -> usize {
        self.get(key).v()
    }

    pub fn vv(&self, key: &str) -> Option<usize> {
        self.get(key).vv()
    }

    pub fn inc(&mut self, key: &str) -> usize {
        self.get_mut(key).inc()
    }
    pub fn dec(&mut self, key: &str) -> usize {
        self.get_mut(key).dec()
    }

    pub fn add(&mut self, key: &str, n: usize) -> usize {
        self.get_mut(key).add(n)
    }
    pub fn sub(&mut self, key: &str, n: usize) -> usize {
        self.get_mut(key).sub(n)
    }
}
