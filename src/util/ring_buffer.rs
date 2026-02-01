/// Fixed-size circular buffer for time-series data.
pub struct RingBuffer<T> {
    data: Vec<Option<T>>,
    capacity: usize,
    write_pos: usize,
    len: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![None; capacity],
            capacity,
            write_pos: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.data[self.write_pos] = Some(value);
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    pub fn latest(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = if self.write_pos == 0 {
            self.capacity - 1
        } else {
            self.write_pos - 1
        };
        self.data[idx].as_ref()
    }

    /// Returns items in chronological order (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start = if self.len < self.capacity {
            0
        } else {
            self.write_pos
        };
        (0..self.len).filter_map(move |i| {
            let idx = (start + i) % self.capacity;
            self.data[idx].as_ref()
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Convert to a Vec of (x, y) pairs for ratatui Chart datasets.
    /// x is the sample index (0 = oldest), y is the value mapped by the closure.
    pub fn as_dataset<F: Fn(&T) -> f64>(&self, map_fn: F) -> Vec<(f64, f64)> {
        self.iter()
            .enumerate()
            .map(|(i, v)| (i as f64, map_fn(v)))
            .collect()
    }

    /// Like `as_dataset` but only returns the last `n` samples.
    /// x values are 0..n (re-indexed so the window always starts at 0).
    pub fn as_dataset_last_n<F: Fn(&T) -> f64>(&self, n: usize, map_fn: F) -> Vec<(f64, f64)> {
        let items: Vec<&T> = self.iter().collect();
        let skip = items.len().saturating_sub(n);
        items[skip..]
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, map_fn(v)))
            .collect()
    }
}
