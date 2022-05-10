package com.sample;

import org.apache.kafka.clients.admin.AdminClient;
import org.apache.kafka.clients.admin.CreateTopicsResult;
import org.apache.kafka.clients.admin.KafkaAdminClient;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.producer.ProducerConfig;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.*;
import java.util.concurrent.ExecutionException;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public class BenchmarkInitializer {

    private static final String KAFKA_ENV_PREFIX = "KAFKA_";
    private final Logger logger = LoggerFactory.getLogger(BenchmarkInitializer.class);
    private final Properties properties;

    public static void main(String[] args) throws InterruptedException, ExecutionException {
        BenchmarkInitializer benchmarkProducer = new BenchmarkInitializer();
        benchmarkProducer.start();
    }

    public BenchmarkInitializer()  {
        this.properties = buildProperties(defaultProps, System.getenv(), KAFKA_ENV_PREFIX);
    }

    private void start() throws ExecutionException, InterruptedException {
        logger.info("Initializing topics for the benchmark");
        final String topicPrefix = System.getenv().getOrDefault("TOPIC_PREFIX", "sample");

        final Short nbTopics = Short.valueOf(System.getenv().getOrDefault("NB_TOPICS", "1"));
        final List<String> topicsNames = IntStream.range(0, nbTopics).mapToObj((e) -> topicPrefix + "_" + e).collect(Collectors.toList());

        final Integer numberOfPartitions = Integer.valueOf(System.getenv().getOrDefault("NUMBER_OF_PARTITIONS", "1"));
        final Short replicationFactor = Short.valueOf(System.getenv().getOrDefault("REPLICATION_FACTOR", "1"));

        try (AdminClient adminClient = KafkaAdminClient.create(properties)) {
            createTopics(adminClient, topicsNames, numberOfPartitions, replicationFactor);

            logger.info("All topics are initialized");
        }

    }


    private Map<String, String> defaultProps = Map.of(
            ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092"
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

    private void createTopics(AdminClient adminClient, List<String> topicsNames, Integer numberOfPartitions, Short replicationFactor) throws ExecutionException, InterruptedException {
        Set<String> existingTopics = adminClient.listTopics().names().get();
        logger.info("Existings topics: {}", existingTopics.toString());
        List<String> topicsToCreate = topicsNames.stream()
                .filter((topic) -> !existingTopics.contains(topic))
                .collect(Collectors.toList());

        List<NewTopic> newTopics = topicsToCreate.stream()
                .map((topic) -> new NewTopic(topic, numberOfPartitions, replicationFactor))
                .collect(Collectors.toList());
        try {
            logger.info("Creating topics {} with {} partions and replication factor {}", topicsToCreate.toString(), numberOfPartitions, replicationFactor);
            CreateTopicsResult topicsCreationResult = adminClient.createTopics(newTopics);
            topicsCreationResult.all().get();
        } catch (ExecutionException e) {
            //silent ignore if topic already exists
        } catch (InterruptedException e) {
            // irrelevant for the bench
            throw new RuntimeException(e);
        }
    }

}
