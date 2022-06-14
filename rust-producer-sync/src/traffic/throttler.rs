use std::collections::vec_deque::VecDeque;
use std::time::{Duration, Instant};
use crate::traffic::{Rate, TrafficGen};

pub(crate) struct Throttler {
    rate: Rate,
    queue: VecDeque<Instant>
}

impl Throttler {

    pub(crate) fn new(rate: Rate) -> Self {
        let amount = rate.amount as usize;
        Throttler {
            rate,
            queue: VecDeque::with_capacity(amount)
        }
    }

    fn free_up(&mut self) {
        let now = Instant::now();
        let threshold = now - self.rate.duration;
        while let Some(frt) = self.queue.front() {
            if *frt < threshold { // no longer relevant to be counted, can be removed
                self.queue.pop_front();
            } else { // oldest element is still relevant to count in bucket, can't free up more space
                break;
            }
        }
    }

}

impl TrafficGen for Throttler {

    fn next_wait(&mut self) -> Duration {
        let now = Instant::now();
        self.free_up();
        self.queue.push_back(now);
        if (self.queue.len() as u32) < self.rate.amount {
            return Duration::from_secs(0);
        }
        let fst = self.queue.front().unwrap();
        (*fst + self.rate.duration) - now
    }

    fn gen_arrivals(&mut self, nb_to_gen: usize) -> Vec<(Duration, Instant)> {
        let step = self.rate.duration / self.rate.amount as u32;
        let mut res = Vec::with_capacity(nb_to_gen);
        let mut origin = Instant::now();
        for _ in 0..nb_to_gen {
            origin += step;
            res.push((step, origin));
        }
        res
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use itertools::Itertools;
    use crate::traffic::{Rate, TrafficGen};
    use crate::traffic::throttler::Throttler;

    #[test]
    fn print_wait_distribution() {
        // not a test per-se (and hard to turn into one, but prints the distribution in CSV to build an histogram)
        let mut i = 0;
        let amount_per_sec = 750;
        let rate = Rate { amount: amount_per_sec, duration: Duration::from_secs(1) };
        let mut gen = Throttler::new(rate);
        let mut event_secs_buckets = HashMap::new();
        let origin = Instant::now();
        while i < 30_000 { // 30k msg at 750msg/s should be ~>30s to run the test
            let wait = gen.next_wait();
            if wait > Duration::from_secs(0) {
                sleep(wait);
            }
            let time = Instant::now();
            let bucket_seconds = time.duration_since(origin).as_secs();
            let count = event_secs_buckets.entry(bucket_seconds).or_insert(0);
            *count += 1;
            i += 1;
        }
        let counts_per_sec = event_secs_buckets.iter()
            .sorted()
            .dropping(1) // first and last may be outliers
            .dropping_back(1);
        for (_, count) in counts_per_sec {
            assert_eq!(amount_per_sec, *count);
        }
    }

    #[test]
    fn print_instant_distribution() {
        // not a test per-se (and hard to turn into one, but prints the distribution in CSV to build an histogram)
        let nb_to_gen = 100_000_000;
        let amount_per_sec = 750;
        let rate = Rate { amount: amount_per_sec, duration: Duration::from_secs(1) };
        let mut gen = Throttler::new(rate);
        let mut event_secs_buckets = HashMap::new();
        let origin = Instant::now();
        let arrivals = gen.gen_arrivals(nb_to_gen);
        for (_, arrival) in arrivals {
            let bucket_seconds = arrival.duration_since(origin).as_secs();
            let count = event_secs_buckets.entry(bucket_seconds).or_insert(0);
            *count += 1;
        }
        let counts_per_sec = event_secs_buckets.iter()
            .sorted()
            .dropping(1) // first and last may be outliers
            .dropping_back(1);
        for (_, count) in counts_per_sec {
            assert_eq!(amount_per_sec, *count);
        }
    }

}