use std::time::{Duration, Instant};

pub(crate) mod poisson;
pub(crate) mod throttler;

#[derive(Debug)]
/// A struct to model a Rate limit:
///     amount: maximum number of events over the specified
///     duration
pub(crate) struct Rate {
    pub(crate) amount: u32,
    pub(crate) duration: Duration
}

/// A "TrafficGen" is generating to generate events in respect to a RateLimit
///     * Either by generating an array of "arrival instants" (when should the events occur, theoretically
///     * Or by keeping a local state and telling the caller how long it should wait before sending the next event
pub(crate) trait TrafficGen {
    fn gen_arrivals(&mut self, nb_to_gen: usize) -> Vec<(Duration, Instant)>;
    fn next_wait(&mut self) -> Duration;
}