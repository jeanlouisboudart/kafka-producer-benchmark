package com.sample;

import java.time.Instant;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Properties;
import java.util.Timer;
import java.util.TimerTask;
import java.util.UUID;

import org.apache.commons.lang3.time.DurationFormatUtils;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.Producer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.clients.producer.RecordMetadata;
import org.apache.kafka.common.Metric;
import org.apache.kafka.common.MetricName;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Injector {

    private final Logger logger = LoggerFactory.getLogger(Injector.class);
    private final Properties properties;

    private final Short messageSize;
    private final Long nbMessages;
    private final Long reportingInterval;
    private final List<String> topicNames;


    public Injector(Properties properties, List<String> topicNames, Long nbMessages, Short messageSize, Long reportingInterval) {
        this.properties = properties;
        this.messageSize = messageSize;
        this.reportingInterval = reportingInterval;
        this.topicNames = topicNames;
        this.nbMessages = nbMessages;
    }


    public void start() {
        logger.info("Running benchmark with {} topics {} messages of {} bytes each", topicNames.size(), nbMessages, messageSize);

        try (Producer<String, String> producer = new KafkaProducer<>(properties)) {
            Instant startTime = Instant.now();
            Timer timer = configureMetricsCollector(producer);
            long totalMsgs = 0;
            int nbTopics = topicNames.size();
            //Map<String, List<ProducerRecord>> toSend = new HashMap<>();

            while (totalMsgs <= nbMessages) {
                //simulate high cardinality in the key
                String key = UUID.randomUUID().toString();
                String value = randomPayload(messageSize);
                String topicName = topicNames.get((int)(totalMsgs % nbTopics));

                ProducerRecord<String, String> record = new ProducerRecord<>(topicName, key, value);
                producer.send(record, (recordMetadata, exception) -> sendCallback(record, recordMetadata, exception));
                /*
                toSend.computeIfAbsent(topicName, k -> new ArrayList<>()).add(record);
                if (totalMsgs % 1000 == 0) {
                    toSend.values().forEach(perTopic -> perTopic.forEach(msg->producer.send(record, (recordMetadata, exception) -> sendCallback(record, recordMetadata, exception))));
                }
                */
                totalMsgs++;

            }
            //flush remaining messages before closing the bench
            producer.flush();
            printFinalMetrics(producer, timer, startTime);
        }
    }

    private Timer configureMetricsCollector(Producer<String, String> producer) {
        Timer timer = new Timer();
        timer.schedule(new TimerTask() {
            @Override
            public void run() {
                printMetrics(producer);
            }
        }, 0, reportingInterval);
        return timer;
    }

    private void printMetrics(Producer<String, String> producer) {
        Map<MetricName, ? extends Metric> metrics = producer.metrics();
        double totalMsgsMetric = producerMetric(metrics, "record-send-total");
        double avgSendRate = producerMetric(metrics, "record-send-rate");
        double queueTimeAvg = producerMetric(metrics, "record-queue-time-avg");
        double batchSizeAvg = producerMetric(metrics, "batch-size-avg");
        double requestRate = producerMetric(metrics, "request-rate");
        double requestLatencyAvg = producerMetric(metrics, "request-latency-avg");
        double recordsPerRequestAvg = producerMetric(metrics, "records-per-request-avg");
        logger.info("Sent rate = {}/sec, duration spent in queue = {}ms, batch size = {}, request rate = {}/sec, request latency avg = {}ms, records per ProduceRequest = {}", avgSendRate, queueTimeAvg, batchSizeAvg, requestRate, requestLatencyAvg, recordsPerRequestAvg);
    }

    private void printFinalMetrics(Producer<String, String> producer, Timer timer, Instant startTime) {
        //we need to explicitly print the metrics as we have stopped the timer
        timer.cancel();
        printMetrics(producer);
        Map<MetricName, ? extends Metric> metrics = producer.metrics();
        double totalMsgsMetric = producerMetric(metrics, "record-send-total");
        double requestTotal = producerMetric(metrics, "request-total");
        String duration = DurationFormatUtils.formatDurationHMS(Instant.now().toEpochMilli() - startTime.toEpochMilli());
        logger.info("REPORT: Produced {} with {} ProduceRequest in {}", totalMsgsMetric, requestTotal, duration);
    }

    private void sendCallback(ProducerRecord<String, String> record, RecordMetadata recordMetadata, Exception e) {
        if (e != null) {
            logger.error("failed sending in " + record.topic() + " key: " + record.key(), e);
        }
    }

    private static String randomPayload(int size) {
        char[] buf = new char[size];
        Arrays.fill(buf, 'j');
        return new String(buf);
    }

    private static double producerMetric(Map<MetricName, ? extends Metric> metrics, String metric) {
        return metrics
                .entrySet()
                .stream()
                .filter(e -> e.getKey().name().equals(metric) && e.getKey().group().equals("producer-metrics"))
                .mapToDouble(e -> (double) e.getValue().metricValue())
                .sum();
    }

}
