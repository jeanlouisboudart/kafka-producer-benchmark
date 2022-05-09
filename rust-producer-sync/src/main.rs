use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Utc;
use log::{debug, error, info};
use rand::prelude::SliceRandom;
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use uuid::Uuid;
use crate::stats::{LoggingContext, StatsCheckpoint};

pub(crate) mod config;
pub(crate) mod stats;

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

fn main() {
    env_logger::init();
    let mut rng = rand::thread_rng();
    let (kafka_conf, benchmark_conf) = config::conf_from_env();
    let mut records_sent = 0;
    let msgs = random_messages::<100>(benchmark_conf.message_size);
    let topic_names = benchmark_conf.topic_names();
    let stats_arc = Arc::new(Mutex::new(StatsCheckpoint::default()));
    let stats = LoggingContext::new(stats_arc.clone());
    let producer: BaseProducer<LoggingContext> = kafka_conf
        .create_with_context::<LoggingContext, BaseProducer<LoggingContext>>(stats)
        .expect("Could not create Kafka producer");

    let start = Utc::now();
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
                debug!("Queue is full, flushing");
                producer.poll(Duration::from_millis(500));
            } else {
                error!("Got err: {:?}", err);
            }
        }
        records_sent+= 1;
    }
    // flush outstanding messages before finishing the benchmark
    while producer.in_flight_count() > 0 {
        info!("Flushing outstanding messages");
        producer.flush(Duration::from_secs(10));
    }
    let elapsed = Utc::now() - start;
    let stats = stats_arc.lock().unwrap();
    let seconds = elapsed.num_seconds();
    info!(
        "REPORT: Produced {} with {} ProduceRequest in {:.2}:{:.2}:{:.2}.{:.3}",
        stats.nb_msgs_sent,
        stats.request_count,
        seconds / 60 / 60,
        seconds / 60 % 60,
        seconds % 60,
        elapsed.num_milliseconds() % (seconds * 1000)
    )
}

fn random_messages<const NB: usize>(msg_size: usize) -> [String;NB] {
    [msg_size; NB].map(random_msg)
}

fn random_msg(msg_size: usize) -> String {
    random_string::generate(msg_size, CHARSET)
}