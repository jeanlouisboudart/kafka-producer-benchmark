use std::time::Duration;
use rand::distributions::Uniform;
use rand::Rng;

pub(crate) struct PoissonGen<T: Rng> {
    rng: T,
    rate_per_sec: f64
}

impl<T: Rng> PoissonGen<T> {
    pub(crate) fn new(rng: T, rate: f64) -> Self {
        Uniform::new_inclusive(0.0, 1.0);
        PoissonGen { rng, rate_per_sec: rate }
    }

    pub(crate) fn next_wait(&mut self) -> Duration {
        // https://stackoverflow.com/questions/9832919/generate-poisson-arrival-in-java
        let rate_per_ms = self.rate_per_sec / 1_000.0;
        let uniform_incl_0_excl_1 = self.rng.gen::<f64>(); // by default: uniformly distributed in [0, 1)
        let uniform_incl_1_excl_0 = 1.0 - uniform_incl_0_excl_1; // 1 - rand() => uniformly distributed in (0, 1]
        let wait_ms = -1.0 * f64::ln(uniform_incl_1_excl_0) / rate_per_ms; // -ln(U) / rate
        Duration::from_millis(wait_ms as u64)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Instant;
    use itertools::Itertools;
    use rand::thread_rng;
    use crate::gen::PoissonGen;

    #[test]
    fn test_distrib() {
        let mut i = 0;
        let rate_per_sec = 200.0; // 200 events per sec
        let mut gen = PoissonGen::new(thread_rng(), rate_per_sec);
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
}

