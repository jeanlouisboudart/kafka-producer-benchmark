use std::env;
use std::str::FromStr;
use rdkafka::ClientConfig;

const KAFKA_CONF_PREFIX: &str = "KAFKA_";

pub(crate) struct BenchmarkConfig {
    pub(crate) topic_prefix: String,
    pub(crate) message_size: usize,
    pub(crate) reporting_interval: u32,
    pub(crate) number_of_messages: u32,
    pub(crate) use_random_keys: bool,
    pub(crate) nb_topics: usize,
    pub(crate) agg_per_topic_buffer_size: usize,
}

impl BenchmarkConfig {
    fn default() -> BenchmarkConfig {
        BenchmarkConfig {
            topic_prefix: "sample".to_string(),
            message_size: 200,
            reporting_interval: 1000,
            number_of_messages: 1000000,
            use_random_keys: true,
            nb_topics: 1,
            agg_per_topic_buffer_size: 1,
        }
    }

    pub(crate) fn topic_names(&self) -> Vec<String> {
        let prefix = self.topic_prefix.clone();
        (0..self.nb_topics)
            .map(|i| format!("{prefix}_{i}"))
            .collect()
    }
}

pub(crate) fn conf_from_env() -> (ClientConfig, BenchmarkConfig) {
    let benchmark_conf = benchmark_conf_from_env();
    let mut kafka_conf = kafka_conf_from_env();
    kafka_conf.set("statistics.interval.ms", benchmark_conf.reporting_interval.to_string());
    (kafka_conf, benchmark_conf)
}

fn kafka_conf_from_env() -> ClientConfig {
    let mut conf = ClientConfig::default();
    for (name, value) in env::vars() {
        if let Some(key) = name.strip_prefix(KAFKA_CONF_PREFIX) {
            conf.set(key.replace('_', ".").to_lowercase(), value);
        }
    }
    conf
}

fn benchmark_conf_from_env() -> BenchmarkConfig {
    let mut conf = BenchmarkConfig::default();
    if let Ok(prefix) = env::var("TOPIC_PREFIX") {
        conf.topic_prefix = prefix
    }
    if let Ok(msg_size) = env::var("MESSAGE_SIZE") {
        conf.message_size = usize::from_str(msg_size.as_str()).expect("MESSAGE_SIZE env. variable doesn't seem to be a valid usize integer")
    }
    if let Ok(interval) = env::var("REPORTING_INTERVAL") {
        conf.reporting_interval = u32::from_str(interval.as_str()).expect("REPORTING_INTERVAL env. variable doesn't seem to be a valid u32 integer")
    }
    if let Ok(nb_msg) = env::var("NB_MESSAGES") {
        conf.number_of_messages = u32::from_str(nb_msg.as_str()).expect("NB_MESSAGES env. variable doesn't seem to be a valid u32 integer")
    }
    if let Ok(rnd_keys) = env::var("USE_RANDOM_KEYS") {
        conf.use_random_keys = bool::from_str(rnd_keys.as_str()).expect("REPORTING_INTERVAL env. variable doesn't seem to be a valid boolean value")
    }
    if let Ok(nb_topics) = env::var("NB_TOPICS") {
        conf.nb_topics = usize::from_str(nb_topics.as_str()).expect("NB_TOPICS env. variable doesn't seem to be a valid usize integer")
    }
    if let Ok(nb_msgs_aggd_per_topic) = env::var("AGG_PER_TOPIC_NB_MESSAGES") {
        conf.agg_per_topic_buffer_size = usize::from_str(nb_msgs_aggd_per_topic.as_str()).expect("AGG_PER_TOPIC_NB_MESSAGES doesn't seem to be a valid usize integer");
    }
    conf
}

