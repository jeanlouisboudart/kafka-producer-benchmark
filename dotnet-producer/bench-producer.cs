using Confluent.Kafka;
using System;
using System.Collections.Generic;
using System.Linq;
using Newtonsoft.Json.Linq;
using Microsoft.Extensions.Logging;

class BenchProducer<K,V>: IDisposable {

    private static ILogger logger = Logger.GetLogger(typeof(BenchProducer<K,V>));

    private readonly Confluent.Kafka.IProducer<K,V> producer;

    public double lastRequestCount = 0;
    public double lastTotalMsgsMetric = 0;
    public double lastMetricCollectionTimestamp = 0;

    public BenchProducer(IEnumerable<KeyValuePair<string, string>> config) {
        producer = new ProducerBuilder<K, V>(config)
            .SetStatisticsHandler((_, jsonAsString) => handleStats(jsonAsString))
            .Build();
    }

    public void produce(string topicName, K key, V value) {
        while (true) {
            try {
                producer.Produce(
                            topicName,
                            new Message<K, V> { Key = key, Value = value },
                            (deliveryReport) => handleDeliveryReport(deliveryReport)
                );
                
            } catch (ProduceException<K, V> e) {
                if (e.Error.Code == ErrorCode.Local_QueueFull) {
                    logger.LogDebug("Produce message queue full, waiting for deliveries");
                    producer.Poll(TimeSpan.FromMilliseconds(500));
                    continue;
                } else {
                    logger.LogError(e, "Produce message failed for message with {key}", key);
                    throw;
                }

            }
            break;
        }
        producer.Poll(TimeSpan.FromSeconds(0));
    }


    private void handleDeliveryReport(DeliveryReport<K,V> deliveryReport) {
        if (deliveryReport.Error.Code != ErrorCode.NoError) {
            logger.LogError("Failed to deliver message: {error}", deliveryReport.Error.Reason);
        }
    }

private void handleStats(string jsonAsString) {
        var stats = JObject.Parse(jsonAsString);

        long currentNbMessageSent = (long)stats["txmsgs"];

        // we need to convert into seconds in order to compute rate per sec
        var currentTs = ((long)stats["ts"]) / 1000 / 1000; 
        try {
            var batchSizeAvgList = stats["topics"]
                .Values()
                .Select((e) => (int)e["batchsize"]["avg"])
                .ToList();
            var batchSizeAvg = batchSizeAvgList.Count() > 0 ? batchSizeAvgList.Average(): 0;

            var queueLatencyAvgList = stats["brokers"]
                .Values()
                .Select((e) => (int)e["int_latency"]["avg"])
                .ToList();
            var queueTimeAvg = queueLatencyAvgList.Count() > 0 ? queueLatencyAvgList.Average() / 1000 : 0;

            var requestLatencyAvgList = stats["brokers"]
                .Values()
                .Select((e) => (int)e["rtt"]["avg"])
                .ToList();
            var requestLatencyAvg = requestLatencyAvgList.Count() > 0 ? requestLatencyAvgList.Average() / 1000: 0;

            var requestCountList = stats["brokers"]
                .Values()
                .Select((e) => (int)e["req"]["Produce"])
                .ToList();
            var requestCount = requestCountList.Count() > 0 ? requestCountList.Average() : 0;

            var elapsed = Math.Max(currentTs - lastMetricCollectionTimestamp, 0);
            var requestRate = elapsed > 0 ? (requestCount - lastRequestCount) / elapsed : requestCount;
            var diffMessageSent = (currentNbMessageSent - lastTotalMsgsMetric);
            var nbMessageSentPerSec = elapsed > 0 ? diffMessageSent / elapsed : currentNbMessageSent;

            var recordsPerRequestAvg = nbMessageSentPerSec > 0 ? nbMessageSentPerSec / requestRate : 0;

            lastMetricCollectionTimestamp = currentTs;
            lastTotalMsgsMetric = currentNbMessageSent;
            lastRequestCount = requestCount;
            logger.LogInformation("Sent rate = {nbMessageSentPerSec}/sec, duration spent in queue = {queueTimeAvg}ms, batch size = {batchSizeAvg}, request rate = {requestRate}/sec, request latency avg = {requestLatencyAvg}ms, records per ProduceRequest = {recordsPerRequestAvg}", nbMessageSentPerSec, queueTimeAvg, batchSizeAvg, requestRate, requestLatencyAvg, recordsPerRequestAvg);

        } catch (Exception e) {
            logger.LogError(e, "Something wrong occured while processing statistics");
        }

    }

    public void Flush() {
        producer.Flush(TimeSpan.FromSeconds(10));
    }

    public void Dispose() {
        producer.Dispose();
    }
}