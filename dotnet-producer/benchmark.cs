using Confluent.Kafka;
using System;
using System.Collections.Generic;
using System.Collections;
using System.Linq;
using System.Diagnostics;

class ProducerBenchmark {

    private readonly short messageSize;
    private readonly long reportingInterval;
    private readonly long nbMessages;
    private readonly bool useRandomKeys;
    private readonly short aggregatePerTopicNbMessages;
    private readonly IList<string> topicNames;

    private const string KAFKA_PREFIX="KAFKA_";
    public ProducerBenchmark() {
        string topicPrefix = Utils.GetEnvironmentVariable("TOPIC_PREFIX","sample");
        messageSize = Convert.ToInt16(Utils.GetEnvironmentVariable("MESSAGE_SIZE", "200"));
        reportingInterval = Convert.ToInt64(Utils.GetEnvironmentVariable("REPORTING_INTERVAL", "1000"));
        nbMessages = Convert.ToInt64(Utils.GetEnvironmentVariable("NB_MESSAGES", "1000000"));
        useRandomKeys = Convert.ToBoolean(Utils.GetEnvironmentVariable("USE_RANDOM_KEYS", "true")); 

        aggregatePerTopicNbMessages = Convert.ToInt16(Utils.GetEnvironmentVariable("AGG_PER_TOPIC_NB_MESSAGES", "1"));
        short nbTopics = Convert.ToInt16(Utils.GetEnvironmentVariable("NB_TOPICS", "1"));
        topicNames = Enumerable.Range(0, nbTopics).Select(x => topicPrefix + "_" + x).ToList();
        Console.WriteLine($"Running benchmark with {topicNames.Count()} topics {nbMessages} messages of {messageSize} bytes each with random keys={useRandomKeys}");
        if (aggregatePerTopicNbMessages > 1) {
             Console.WriteLine($"Will use grouping per topic and bulk send every {aggregatePerTopicNbMessages} messages");
        }

    }

    public void start() {

        // Prepare a bunch of messages
        int nbFakeData = topicNames.Count() * 1000;
        IList<String> randomMessages = Enumerable.Range(0, nbFakeData)
                            .Select(x => Utils.RandomString(messageSize))
                            .ToList();
        IList<String> randomKeys = Enumerable.Range(0, nbFakeData)
                            .Select(x => Guid.NewGuid().ToString())
                            .ToList();

        Stopwatch timer = Stopwatch.StartNew();

        using (var producer = new BenchProducer<string,string>(buildProperties())) {
            var totalMsgs = 0;
            for (int i = 0; i < nbMessages; ++i) {
                string key = useRandomKeys ? randomKeys[totalMsgs % nbFakeData] : null;
                string value = randomMessages[totalMsgs % nbFakeData];
                //write sequentially into topics to make it deterministic and simulate load with high cardinality
                string topicName = topicNames[totalMsgs % topicNames.Count];
                producer.produce(topicName,key,value);
                totalMsgs++;
            }
            producer.Flush();
            timer.Stop();
            TimeSpan duration = timer.Elapsed;
            string durationAsString = String.Format("{0:00}:{1:00}:{2:00}.{3}", duration.Hours, duration.Minutes, duration.Seconds, duration.Milliseconds);
            Console.WriteLine($"REPORT: Produced {producer.lastTotalMsgsMetric} with {producer.lastRequestCount} ProduceRequests in {durationAsString}");

        }
    }

    private ProducerConfig buildProperties() {
        var map = new Dictionary<string,string>();
        foreach (DictionaryEntry entry in Environment.GetEnvironmentVariables()) {
            if (entry.Key.ToString().StartsWith(KAFKA_PREFIX)) {
                var k = entry.Key.ToString().Replace(KAFKA_PREFIX,"").ToLower().Replace("_",".");
                map.Add(k, entry.Value.ToString());
            }
        }
        //default config if not already set
        map.TryAdd("acks", "all");
        map.TryAdd("bootstrap.servers","localhost:9092");
        map.TryAdd("statistics.interval.ms", reportingInterval.ToString());
        return new ProducerConfig(map);
    }

    public static void Main(string[] args) {
        ProducerBenchmark benchmark = new ProducerBenchmark();
        benchmark.start();
    }

}