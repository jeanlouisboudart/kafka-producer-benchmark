using Confluent.Kafka;
using System;
using System.Collections.Generic;
using System.Collections;
using System.Linq;
using System.Diagnostics;
using Newtonsoft.Json.Linq;


class BenchProducer<K,V>: IDisposable {

    private readonly Confluent.Kafka.IProducer<K,V> producer;

    public double lastRequestCount = 0;
    public double lastTotalMsgsMetric = 0;
    public double lastMetricCollectionTimestamp = 0;

    public BenchProducer(IEnumerable<KeyValuePair<string, string>> config) {
        producer = new ProducerBuilder<K, V>(config)
            .SetStatisticsHandler((_, jsonAsString) => handleStats(jsonAsString))
            .Build();
    }

    public void produce(String topicName, K key, V value) {
        while (true) {
            try {
                producer.Produce(
                            topicName,
                            new Message<K, V> { Key = key, Value = value },
                            (deliveryReport) => handleDeliveryReport(deliveryReport)
                );
                
            } catch (ProduceException<String, String> e) {
                if (e.Error.Code == ErrorCode.Local_QueueFull) {
                    producer.Poll(TimeSpan.FromMilliseconds(500));
                    continue;
                } else {
                    throw;
                }

            }
            break;
        }
        producer.Poll(TimeSpan.FromSeconds(0));
    }


    private void handleDeliveryReport(DeliveryReport<K,V> deliveryReport) {
        if (deliveryReport.Error.Code != ErrorCode.NoError) {
            Console.WriteLine($"Failed to deliver message: {deliveryReport.Error.Reason}");
        }
    }

private void handleStats(String jsonAsString) {
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
            Console.WriteLine($"Sent rate = {nbMessageSentPerSec}/sec, duration spent in queue = {queueTimeAvg}ms, batch size = {batchSizeAvg}, request rate = {requestRate}/sec, request latency avg = {requestLatencyAvg}ms, records per ProduceRequest = {recordsPerRequestAvg}");

        } catch (Exception e) {
            Console.WriteLine("Something wrong occured while processing statistics :" + e);
        }

    }

    public void Flush() {
        producer.Flush(TimeSpan.FromSeconds(10));
    }

    public void Dispose() {
        producer.Dispose();
    }
}