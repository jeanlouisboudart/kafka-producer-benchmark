use std::time::{Duration, Instant};
use rand::Rng;
use crate::traffic::{Rate, TrafficGen};

pub struct PoissonGen<T: Rng> {
    rng: T,
    rate: Rate
}

impl<T: Rng> PoissonGen<T> {
    pub(crate) fn new(rng: T, rate: Rate) -> Self {
        PoissonGen { rng, rate }
    }
}


impl<T: Rng> TrafficGen for PoissonGen<T> {

    fn next_wait(&mut self) -> Duration {
        // https://stackoverflow.com/questions/9832919/generate-poisson-arrival-in-java
        let rate_per_ms = self.rate.amount as f64 / self.rate.duration.as_millis() as f64;
        let uniform_incl_0_excl_1 = self.rng.gen::<f64>(); // by default: uniformly distributed in [0, 1)
        let uniform_incl_1_excl_0 = 1.0 - uniform_incl_0_excl_1; // 1 - rand() => uniformly distributed in (0, 1]
        let wait_ms = -1.0 * f64::ln(uniform_incl_1_excl_0) / rate_per_ms; // -ln(U) / rate
        Duration::from_millis(wait_ms as u64)
    }

    fn gen_arrivals(&mut self, nb_to_gen: usize) -> Vec<(Duration, Instant)> {
        let mut generated = 0;
        let mut res = Vec::with_capacity(nb_to_gen);
        let mut arrival = Instant::now();
        while generated < nb_to_gen {
            let should_wait_for = self.next_wait();
            arrival += should_wait_for;
            res.push((should_wait_for, arrival));
            generated+= 1;
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
    use rand::thread_rng;
    use crate::traffic::poisson::PoissonGen;
    use crate::traffic::{Rate, TrafficGen};

    #[test]
    fn print_wait_distribution() {
        // not a test per-se (and hard to turn into one, but prints the distribution in CSV to build an histogram)
        let mut i = 0;
        let rate = Rate { amount: 200, duration: Duration::from_secs(1) };
        let mut gen = PoissonGen::new(thread_rng(), rate);
        let mut event_secs_buckets = HashMap::new();
        let origin = Instant::now();
        while i < 10_000 {
            sleep(gen.next_wait());
            let time = Instant::now();
            let bucket_seconds = time.duration_since(origin).as_secs();
            let count = event_secs_buckets.entry(bucket_seconds).or_insert(0);
            *count += 1;
            i += 1;
        }
        for (secs, count) in event_secs_buckets.iter().sorted() {
            println!("{},{}", secs, count);
        }
    }

    #[test]
    fn print_instant_distribution() {
        // not a test per-se (and hard to turn into one, but prints the distribution in CSV to build an histogram)
        let nb_to_gen = 1_000_000;
        let rate = Rate { amount: 200, duration: Duration::from_secs(1) };
        let mut gen = PoissonGen::new(thread_rng(), rate);
        let mut event_secs_buckets = HashMap::new();
        let origin = Instant::now();
        let arrivals = gen.gen_arrivals(nb_to_gen);
        for (_, arrival) in arrivals {
            let bucket_seconds = arrival.duration_since(origin).as_secs();
            let count = event_secs_buckets.entry(bucket_seconds).or_insert(0);
            *count += 1;
        }
        for (secs, count) in event_secs_buckets.iter().sorted() {
            println!("{};{}", secs, count);
        }
    }
}

