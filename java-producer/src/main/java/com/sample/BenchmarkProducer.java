package com.sample;

import org.apache.kafka.clients.producer.ProducerConfig;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.*;
import java.util.concurrent.ExecutionException;
import java.util.stream.Collectors;

public class BenchmarkProducer {

    private static final String KAFKA_ENV_PREFIX = "KAFKA_";
    private final Logger logger = LoggerFactory.getLogger(BenchmarkProducer.class);

    private final Injector injector;


    public static void main(String[] args) throws InterruptedException, ExecutionException {
        BenchmarkProducer benchmarkProducer = new BenchmarkProducer();
        benchmarkProducer.start();
    }

    public BenchmarkProducer() throws ExecutionException, InterruptedException {
        final Properties properties = buildProperties(defaultProps, System.getenv(), KAFKA_ENV_PREFIX);
        this.injector = new Injector(properties);
    }

    private void start() throws InterruptedException {
        logger.info("Running producer benchmark");
        injector.start();

    }


    private Map<String, String> defaultProps = Map.of(
            ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092",
            ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.StringSerializer",
            ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.StringSerializer",
            ProducerConfig.ACKS_CONFIG, "all"
    );

    private Properties buildProperties(Map<String, String> baseProps, Map<String, String> envProps, String prefix) {
        Map<String, String> systemProperties = envProps.entrySet()
                .stream()
                .filter(e -> e.getKey().startsWith(prefix))
                .collect(Collectors.toMap(
                        e -> e.getKey()
                                .replace(prefix, "")
                                .toLowerCase()
                                .replace("_", ".")
                        , e -> e.getValue())
                );

        Properties props = new Properties();
        props.putAll(baseProps);
        props.putAll(systemProperties);
        return props;
    }
}
