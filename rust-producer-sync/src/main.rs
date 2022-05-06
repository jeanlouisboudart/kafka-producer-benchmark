use std::borrow::BorrowMut;
use std::os::macos::raw::stat;
use std::sync::Mutex;
use std::time::Duration;
use log::{error, info, warn};
use rand::prelude::SliceRandom;
use rdkafka::{ClientContext, Statistics};
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use rdkafka::message::DeliveryResult;
use rdkafka::producer::{BaseProducer, BaseRecord, Producer, ProducerContext};
use uuid::Uuid;

pub(crate) mod config;

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

struct StatsCheckpoint {
    request_count: i64,
    nb_msgs_sent: i64,
    last_ts: i64,
}

struct LoggingContext {
    inner: Mutex<StatsCheckpoint>
}

impl LoggingContext {
    fn new() -> Self {
        LoggingContext {
            inner: Mutex::new(StatsCheckpoint {
                request_count: 0,
                nb_msgs_sent: 0,
                last_ts: 0
            })
        }
    }
}

impl ClientContext for LoggingContext {

    fn stats(&self, stats: Statistics) {
        let current_nb_message_sent = stats.txmsgs;
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
        let queue_latency_total = brokers_stats
            .iter()
            .map(|(_, broker)| broker.int_latency.as_ref().map(|w| w.avg).unwrap_or(0))
            .sum::<i64>();
        let queue_latency_avg = queue_latency_total / nb_brokers / 1000;

        let request_latency_total = brokers_stats
            .iter()
            .map(|(_, broker)| broker.rtt.as_ref().map(|w| w.avg).unwrap_or(0))
            .sum::<i64>();
        let request_latency_avg = request_latency_total / nb_brokers / 1000;

        let request_count_total = brokers_stats
            .iter()
            .map(|(_, broker)| broker.req.get("Produce").unwrap_or(&0))
            .sum::<i64>();

        let mut stats_cp = self.inner.lock().unwrap();
        let last_checkpoint = stats_cp.borrow_mut();
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
    fn delivery(&self, _: &DeliveryResult, _: Self::DeliveryOpaque) {}
}

fn main() {
    env_logger::init();
    let mut rng = rand::thread_rng();
    let (kafka_conf, benchmark_conf) = config::conf_from_env();
    let producer: BaseProducer<LoggingContext> = kafka_conf
        .create_with_context::<LoggingContext, BaseProducer<LoggingContext>>(LoggingContext::new())
        .expect("Could not create Kafka producer");
    let mut records_sent = 0;
    let msgs = random_messages::<100>(benchmark_conf.message_size);
    let topic_names = benchmark_conf.topic_names();
    while records_sent < benchmark_conf.number_of_messages {
        let topic = topic_names.get((records_sent % benchmark_conf.nb_topics as u32) as usize).unwrap();
        let payload = msgs.choose(&mut rng).unwrap();
        let record: BaseRecord<String, String> = BaseRecord::to(topic.as_str()).payload(payload);
        let key = Uuid::new_v4().to_hyphenated().to_string();
        let res =
            if benchmark_conf.use_random_keys {
                producer.send(record.key(&key))
            } else {
                producer.send(record)
            };
        if let Err((err, _)) = res {
            if let KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull) = err {
                warn!("Queue is full, flushing");
                producer.poll(Duration::from_millis(500));
            } else {
                error!("Got err: {:?}", err);
            }
        }
        records_sent+= 1;
        if records_sent % 100_000 == 0 {
            info!("Sent {} records", records_sent);
        }
    }
    // flush outstanding messages before finishing the benchmark
    while producer.in_flight_count() > 0 {
        info!("Flushing outstanding messages");
        producer.flush(Duration::from_secs(10));
    }
}

fn random_messages<const NB: usize>(msg_size: usize) -> [String;NB] {
    [msg_size; NB].map(random_msg)
}

fn random_msg(msg_size: usize) -> String {
    random_string::generate(msg_size, CHARSET)
}