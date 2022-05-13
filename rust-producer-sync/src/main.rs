use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Utc;
use itertools::Itertools;
use log::{debug, error, info};
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use rdkafka::message::ToBytes;
use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use uuid::Uuid;
use crate::stats::{LoggingContext, StatsCheckpoint};

pub(crate) mod config;
pub(crate) mod stats;
pub(crate) mod utils;

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

#[derive(Debug)]
struct RecordBuffer<K, V> {
    inner: HashMap<String, Vec<(K, V)>>,
    limit: usize,
    count: usize,
}

impl<K: Clone, V: Clone> RecordBuffer<K, V> {
    fn new(size: usize) -> Self {
        RecordBuffer {
            inner: HashMap::new(),
            limit: size,
            count: 0
        }
    }

    fn drain_if_full(&mut self) -> Option<Vec<(String, (K, V))>> {
        if self.count == 0 || self.count < self.limit {
            return None;
        }
        Some(self.force_drain())
    }

    fn force_drain(&mut self) -> Vec<(String, (K, V))> {
        let res = self.inner
            .clone()
            .into_iter()
            .flat_map(|(topic, recs)|  {
                recs.into_iter()
                    .map(move |rec| (topic.clone(), rec))
            })
            .collect();
        self.inner.clear();
        self.count = 0;
        res
    }

    fn enqueue(&mut self, topic: String, key: K, value: V) {
        self.count += 1;
        let topic_buffer = self.inner
            .entry(topic)
            .or_insert_with(Vec::new);
        topic_buffer.push((key, value));
    }
}

fn main() {
    env_logger::init();
    let (kafka_conf, benchmark_conf) = config::conf_from_env();
    unsafe {
        let config_str: String = utils::kafka_utils::conf_dump(&kafka_conf)
            .unwrap()
            .iter()
            .map(|(k, v)| format!("\t{k}={v}\n"))
            .sorted()
            .collect();
        info!("ClientConfig values: \n {config_str}");
    }
    let mut records_sent = 0;
    let topic_names = benchmark_conf.topic_names();
    let nb_topics = topic_names.len();

    let nb_msgs_to_prep = 100 * nb_topics;
    let msgs = random_messages(nb_msgs_to_prep, benchmark_conf.message_size);
    let keys = random_keys(nb_msgs_to_prep);

    let stats_arc = Arc::new(Mutex::new(StatsCheckpoint::default()));
    let stats = LoggingContext::new(stats_arc.clone());
    let producer: BaseProducer<LoggingContext> = kafka_conf
        .create_with_context::<LoggingContext, BaseProducer<LoggingContext>>(stats)
        .expect("Could not create Kafka producer");
    let mut buffer: RecordBuffer<Option<String>, String> = RecordBuffer::new(benchmark_conf.agg_per_topic_buffer_size);
    let start = Utc::now();
    while records_sent < benchmark_conf.number_of_messages {
        let topic = topic_names.get((records_sent % benchmark_conf.nb_topics) as usize).unwrap();
        let payload = msgs.get(records_sent % nb_msgs_to_prep).unwrap();
        let k = if benchmark_conf.use_random_keys {
            keys.get(records_sent % nb_msgs_to_prep).map(String::clone)
        } else {
            None
        };
        if benchmark_conf.agg_per_topic_buffer_size <= 1 {
            send_until_ok(&producer, topic, &k, &payload);
            records_sent+= 1;
        } else {
            buffer.enqueue(topic.clone(), k, payload.clone());
            for (topic, (key, value)) in buffer.drain_if_full().iter().flatten() {
                send_until_ok(&producer, topic, key, value);
                records_sent += 1;
            }
        }
    }
    if benchmark_conf.agg_per_topic_buffer_size > 1 && buffer.count > 0 { // drain the per-topic buffer one last time
        info!("Draining the 'per-topic' buffer. {} msgs left in buffer. {} already sent", buffer.count, records_sent);
        for (topic, (key, value)) in buffer.force_drain() {
            send_until_ok(&producer, &topic, &key, &value);
            records_sent += 1;
        }
    }
    // flush outstanding messages before finishing the benchmark
    info!("Flushing outstanding messages");
    producer.flush(Duration::from_secs(30));
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
    );
}

fn send_until_ok<'a, K: ToBytes, V: ToBytes>(
    producer: &'a BaseProducer<LoggingContext>,
    topic: &'a str,
    key: &'a Option<K>,
    value: &'a V
) {
    // let key = Uuid::new_v4().to_hyphenated().to_string();
    loop {
        let record = BaseRecord::to(topic).payload(value);
        let res = match key {
            Some(k) =>
                producer.send(record.key(k)),
            None =>
                producer.send(record)
        };
        match res {
            Err((KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull), _)) => {
                debug!("Queue is full, flushing");
                producer.poll(Duration::from_millis(500)); // drain callback queue
            },
            Err((other_err, _)) =>
                error!("Got err: {:?}", other_err),
            _ => // do not retry => break out of loop
                break
        }
    }
    producer.poll(Duration::from_secs(0));
}

fn random_messages(nb: usize, msg_size: usize) -> Vec<String> {
    (0..nb).map(|_| random_msg(msg_size)).collect()
}

fn random_keys(nb: usize) -> Vec<String> {
    (0..nb).map(|_| Uuid::new_v4().to_hyphenated().to_string()).collect()
}

fn random_msg(msg_size: usize) -> String {
    random_string::generate(msg_size, CHARSET)
}


#[cfg(test)]
mod tests {
    use crate::RecordBuffer;

    #[test]
    fn empty_buffer_always_drained() {
        let mut buffer = RecordBuffer::new(0);
        let topic = "t1";
        let key = "k1";
        let value = "v1";
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.to_string(), key.to_string(), value.to_string());
        assert_eq!(Some(vec![(topic.to_string(), (key.to_string(), value.to_string()))]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.to_string(), key.to_string(), value.to_string());
        assert_eq!(Some(vec![(topic.to_string(), (key.to_string(), value.to_string()))]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
    }

    #[test]
    fn one_sized_buffer_always_drained() {
        let mut buffer = RecordBuffer::new(1);
        let topic = "t1";
        let key = "k1";
        let value = "v1";
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.to_string(), key.to_string(), value.to_string());
        assert_eq!(Some(vec![(topic.to_string(), (key.to_string(), value.to_string()))]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.to_string(), key.to_string(), value.to_string());
        assert_eq!(Some(vec![(topic.to_string(), (key.to_string(), value.to_string()))]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
    }

    #[test]
    fn buffer_is_drained_only_when_full() {
        let mut buffer = RecordBuffer::new(2);
        let topic = "t1";
        let key = "k1";
        let key_2 = "k2";
        let value = "v1";
        let value_2 = "v2";
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.to_string(), key.to_string(), value.to_string());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.to_string(), key_2.to_string(), value_2.to_string());
        assert_eq!(Some(vec![(topic.to_string(), (key.to_string(), value.to_string())), (topic.to_string(), (key_2.to_string(), value_2.to_string()))]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
    }

    #[test]
    fn buffer_must_group_per_topic_and_preserve_order() {
        let mut buffer = RecordBuffer::new(6);
        let t1 = "t1";
        let t2 = "t2";
        let t3 = "t3";
        let k1t1 = "k1t1";
        let v1t1 = "v1t1";
        let k2t1 = "k2t1";
        let v2t1 = "v2t1";
        let k1t2 = "k1t2";
        let v1t2 = "v1t2";
        let k2t2 = "k2t2";
        let v2t2 = "v2t2";
        let k1t3 = "k1t3";
        let v1t3 = "v1t3";
        let k2t3 = "k2t3";
        let v2t3 = "v2t3";
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(t1.to_string(), k1t1.to_string(), v1t1.to_string());
        buffer.enqueue(t2.to_string(), k1t2.to_string(), v1t2.to_string());
        buffer.enqueue(t3.to_string(), k1t3.to_string(), v1t3.to_string());
        buffer.enqueue(t1.to_string(), k2t1.to_string(), v2t1.to_string());
        buffer.enqueue(t2.to_string(), k2t2.to_string(), v2t2.to_string());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(t3.to_string(), k2t3.to_string(), v2t3.to_string());

        let drained = buffer.drain_if_full();
        assert!(drained.is_some());
        let mut drained = drained.unwrap();
        drained.sort_by_key(|(t, _)| t.clone());
        assert_eq!(vec![
            (t1.to_string(), (k1t1.to_string(), v1t1.to_string())),
            (t1.to_string(), (k2t1.to_string(), v2t1.to_string())),

            (t2.to_string(), (k1t2.to_string(), v1t2.to_string())),
            (t2.to_string(), (k2t2.to_string(), v2t2.to_string())),

            (t3.to_string(), (k1t3.to_string(), v1t3.to_string())),
            (t3.to_string(), (k2t3.to_string(), v2t3.to_string())),
        ], drained);
    }

}