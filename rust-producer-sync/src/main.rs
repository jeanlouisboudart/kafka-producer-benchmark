use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Utc;
use log::{debug, error, info};
use rand::prelude::SliceRandom;
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use rdkafka::message::ToBytes;
use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use uuid::Uuid;
use crate::stats::{LoggingContext, StatsCheckpoint};

pub(crate) mod config;
pub(crate) mod stats;

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

#[derive(Debug)]
struct RecordBuffer<V: ToBytes> {
    inner: HashMap<String, Vec<V>>,
    limit: usize,
    count: usize,
}

impl<V: ToBytes + Clone> RecordBuffer<V> {
    fn new(size: usize) -> Self {
        RecordBuffer {
            inner: HashMap::new(),
            limit: size,
            count: 0
        }
    }

    fn drain_if_full(&mut self) -> Option<Vec<(String, V)>> {
        if self.count == 0 || self.count < self.limit {
            return None;
        }
        let res = Some(
            self.inner
                .clone()
                .into_iter()
                .flat_map(|(topic, recs)|  {
                    recs.into_iter()
                        .map(move |rec| (topic.clone(), rec))
                })
                .collect()
        );
        self.inner.clear();
        self.count = 0;
        res
    }

    fn enqueue(&mut self, topic: String, value: V) {
        self.count += 1;
        let topic_buffer = self.inner
            .entry(topic)
            .or_insert_with(Vec::new);
        topic_buffer.push(value);
    }
}

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
    let mut buffer: RecordBuffer<String> = RecordBuffer::new(benchmark_conf.agg_per_topic_buffer_size);
    let start = Utc::now();
    while records_sent < benchmark_conf.number_of_messages {
        let topic = topic_names.get((records_sent % benchmark_conf.nb_topics as u32) as usize).unwrap();
        let payload = msgs.choose(&mut rng).unwrap();
        let key = if benchmark_conf.use_random_keys {
            Some(Uuid::new_v4().to_hyphenated().to_string())
        } else {
            None
        };
        if benchmark_conf.agg_per_topic_buffer_size <= 1 {
            send_until_ok(&producer, topic.clone(),  &key, payload.clone());
            records_sent+= 1;
        } else {
            buffer.enqueue(topic.clone(), payload.clone());
            if let Some(records_per_topic) = buffer.drain_if_full() {
                for (topic, record) in records_per_topic {
                    send_until_ok(&producer, topic, &key, record);
                    records_sent += 1;
                }
            }
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
    )
}

fn send_until_ok<K: ToBytes, V: ToBytes>(
    producer: &BaseProducer<LoggingContext>,
    topic: String,
    key: &Option<K>,
    value: V
) {
    // let key = Uuid::new_v4().to_hyphenated().to_string();
    loop {
        let res = match key {
            Some(k) =>
                producer.send(BaseRecord::to(&topic).key(k).payload(&value)),
            None => producer.send::<K, V>(BaseRecord::to(&topic).payload(&value))
        };
        if let Err((err, _)) = res {
            if let KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull) = err {
                debug!("Queue is full, flushing");
                producer.poll(Duration::from_millis(500));
            } else {
                error!("Got err: {:?}", err);
            }
        } else {
            break;
        }
    }
    producer.poll(Duration::from_secs(0));
}

fn random_messages<const NB: usize>(msg_size: usize) -> [String;NB] {
    [msg_size; NB].map(random_msg)
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
        let topic = "t1".to_string();
        let value = "val".to_string();
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.clone(), value.clone());
        assert_eq!(Some(vec![(topic.clone(), value.clone())]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.clone(), value.clone());
        assert_eq!(Some(vec![(topic, value)]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
    }

    #[test]
    fn one_sized_buffer_always_drained() {
        let mut buffer = RecordBuffer::new(1);
        let topic = "t1".to_string();
        let value = "val".to_string();
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.clone(), value.clone());
        assert_eq!(Some(vec![(topic.clone(), value.clone())]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.clone(), value.clone());
        assert_eq!(Some(vec![(topic, value)]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
    }

    #[test]
    fn buffer_is_drained_only_when_full() {
        let mut buffer = RecordBuffer::new(2);
        let topic = "t1".to_string();
        let value = "val".to_string();
        let value_2 = "val2".to_string();
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.clone(), value.clone());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(topic.clone(), value_2.clone());
        assert_eq!(Some(vec![(topic.clone(), value), (topic, value_2)]), buffer.drain_if_full());
        assert_eq!(None, buffer.drain_if_full());
    }

    #[test]
    fn buffer_must_group_per_topic_and_preserve_order() {
        let mut buffer = RecordBuffer::new(6);
        let t1 = "t1";
        let t2 = "t2";
        let t3 = "t3";
        let v1t1 = "v1t1";
        let v2t1 = "v2t1";
        let v1t2 = "v1t2";
        let v2t2 = "v2t2";
        let v1t3 = "v1t3";
        let v2t3 = "v2t3";
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(t1.to_string(), v1t1.to_string());
        buffer.enqueue(t2.to_string(), v1t2.to_string());
        buffer.enqueue(t3.to_string(), v1t3.to_string());
        buffer.enqueue(t1.to_string(), v2t1.to_string());
        buffer.enqueue(t2.to_string(), v2t2.to_string());
        assert_eq!(None, buffer.drain_if_full());
        buffer.enqueue(t3.to_string(), v2t3.to_string());

        assert_eq!(Some(vec![
            (t1.to_string(), v1t1.to_string()),
            (t1.to_string(), v2t1.to_string()),

            (t2.to_string(), v1t2.to_string()),
            (t2.to_string(), v2t2.to_string()),

            (t3.to_string(), v1t3.to_string()),
            (t3.to_string(), v2t3.to_string()),
        ]), buffer.drain_if_full());
    }


}