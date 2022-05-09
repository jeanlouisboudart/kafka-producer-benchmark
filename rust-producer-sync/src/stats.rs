use std::sync::{Arc, Mutex};
use log::{error, info, warn};
use rdkafka::{ClientContext, Statistics};
use rdkafka::producer::{DeliveryResult, ProducerContext};

#[derive(Debug, Default)]
pub(crate) struct StatsCheckpoint {
    pub(crate) request_count: i64,
    pub(crate) nb_msgs_sent: i64,
    pub(crate) last_ts: i64,
}

pub(crate) struct LoggingContext {
    inner: Arc<Mutex<StatsCheckpoint>>
}

impl LoggingContext {
    pub(crate) fn new(m: Arc<Mutex<StatsCheckpoint>>) -> Self {
        LoggingContext {
            inner: m
        }
    }
}

impl ClientContext for LoggingContext {

    fn stats(&self, stats: Statistics) {
        let current_nb_message_sent = stats.txmsgs;
        if current_nb_message_sent == 0 {
            info!("No messages sent, yet. No stats to publish");
            return;
        }
        let current_ts = stats.ts / 1000 / 1000; // we need to convert into seconds in order to compute rate per sec
        let topics_stats = stats.topics;
        let nb_topics = topics_stats.len();
        let batch_size_total = topics_stats
            .iter()
            .map(|(_, topic)| topic.batchsize.avg)
            .sum::<i64>();
        let batch_size_avg = batch_size_total as f64 / nb_topics as f64;

        let brokers_stats = stats.brokers;
        let nb_brokers = brokers_stats.len() as i64;
        if nb_brokers == 0 {
            // Avoiding dividing by 0
            warn!("0 brokers in returned stats");
            return;
        }

        let queue_latency_total = brokers_stats
            .iter()
            .map(|(_, broker)| broker.int_latency.as_ref().map(|w| w.avg).unwrap_or(0))
            .sum::<i64>();
        let queue_latency_avg = (queue_latency_total as f64) / (nb_brokers as f64) / 1000.0;

        let request_latency_total = brokers_stats
            .iter()
            .map(|(_, broker)| broker.rtt.as_ref().map(|w| w.avg).unwrap_or(0))
            .sum::<i64>();
        let request_latency_avg = (request_latency_total as f64)/ (nb_brokers as f64) / 1000.0;

        let request_count_total = brokers_stats
            .iter()
            .map(|(_, broker)| broker.req.get("Produce").unwrap_or(&0))
            .sum::<i64>();

        let stats_arc = self.inner.clone();
        let mut last_checkpoint = stats_arc.lock().unwrap();
        let elapsed = i64::max(current_ts - last_checkpoint.last_ts, 0);
        let request_rate = if elapsed > 0 {
            (request_count_total - last_checkpoint.request_count) as f64 / elapsed as f64
        } else {
            request_count_total as f64
        };
        let diff_message_sent = current_nb_message_sent - last_checkpoint.nb_msgs_sent;
        let nb_message_sent_per_sec = if elapsed > 0 {
            diff_message_sent as f64 / elapsed as f64
        }  else {
            current_nb_message_sent as f64
        };
        let records_per_request_avg = if nb_message_sent_per_sec > 0.0 {
            f64::round(nb_message_sent_per_sec / request_rate * 100.0) as i64 / 100
        } else {
            0
        };
        last_checkpoint.last_ts = current_ts;
        last_checkpoint.nb_msgs_sent = current_nb_message_sent;
        last_checkpoint.request_count = request_count_total;
        info!(
            "Sent rate = {}/sec, duration spent in queue = {}ms, batch size = {}, request rate = {}/sec, request latency avg = {}ms, records per ProduceRequest = {}",
            nb_message_sent_per_sec,
            queue_latency_avg,
            batch_size_avg,
            request_rate,
            request_latency_avg,
            records_per_request_avg
        );
    }
}
impl ProducerContext for LoggingContext {
    type DeliveryOpaque = ();
    fn delivery(&self, res: &DeliveryResult, _: Self::DeliveryOpaque) {
        if let Err((err, _)) = res {
            error!("Could not deliver message: {:?}", err)
        }
    }
}
