import logging
import confluent_kafka
import random
import string
import uuid
import time
import json
import os
from datetime import timedelta
from random import randrange
from threading import Thread

lastRequestCount = 0
lastTotalMsgsMetric = 0
lastMetricCollectionTimestamp = 0

producer = None

log_level = os.getenv("LOG_LEVEL", "INFO")
logging.basicConfig(level=log_level,format="%(asctime)s %(levelname)s %(threadName)s %(name)s %(message)s")

logger = logging.getLogger(__name__)

def main():

    topicPrefix = os.getenv("TOPIC_PREFIX","sample")
    messageSize = int(os.getenv("MESSAGE_SIZE","200"))
    nbMessages= int(os.getenv("NB_MESSAGES", "1000000"))
    reportingInterval= int(os.getenv("REPORTING_INTERVAL", "1000"))
    nbTopics = int(os.getenv("NB_TOPICS", "1"))
    
    logger.info("Running benchmark with %s topics %s messages of %s bytes each", nbTopics, nbMessages, messageSize)

    producer_props = {
        "bootstrap.servers": "localhost:9092",
        "acks": "all"
    }

    producer_props["statistics.interval.ms"] = reportingInterval
    producer_props["stats_cb"] = my_stats_callback

    for key in list(os.environ.keys()):
        if key.startswith("KAFKA_"): 
            producer_props[key.lower().replace("kafka_","").replace("_",".")] = os.environ.get(key)

    global producer
    producer = Producer(producer_props)
    try:


        events = []
        letters = string.ascii_letters

        # Preparing a collection of random events
        for _ in range(100):
            message = ''.join(random.choice(letters) for _ in range(messageSize))
            events.append(message)

        # Producing random events to Kafka
        start_time = time.monotonic()

        totalMsgs= 0

        for _ in range(nbMessages):
            topic = topicPrefix + str(totalMsgs % nbTopics)
            key = str(uuid.uuid4())
            value = random.choice(events)

            producer.produce(topic, key, value)
            
            #if totalMsgs % reportingInterval == 0:
            #    producer.flush()
            totalMsgs+=1

        #flush remaining messages before closing the bench
        producer.flush()

        end_time = time.monotonic()
        logger.info("REPORT: Produced %s with %s ProduceRequest in %s", lastTotalMsgsMetric, lastRequestCount, str(timedelta(seconds=end_time - start_time)))
    except BaseException as e:
        logger.info("Something bad happened : %s", str(e))
    finally:
        logging.info("Shutdown pending")
        producer.close()



def my_stats_callback(stats_json_str):
    stats = json.loads(stats_json_str)
    
    currentNbMessageSent = stats["txmsgs"]
    currentTs = stats["ts"] / 1000 / 1000 # we need to convert into seconds in order to compute rate per sec

    batchSizeAvgList = [value["batchsize"]["avg"] for value in stats["topics"].values()]
    batchSizeAvg = average(batchSizeAvgList) if batchSizeAvgList else 0   

    queueLatencyAvgList = [value["int_latency"]["avg"] for value in stats["brokers"].values()]
    queueTimeAvg = average(queueLatencyAvgList) / 1000 if queueLatencyAvgList else 0

    requestLatencyAvgList = [value["rtt"]["avg"] for value in stats["brokers"].values()]
    requestLatencyAvg = average(requestLatencyAvgList) / 1000 if requestLatencyAvgList else 0

    
    requestCountList = [value["req"]["Produce"] for value in stats["brokers"].values()]
    requestCount = sum(requestCountList) if requestCountList else 0

    global lastRequestCount, lastTotalMsgsMetric, lastMetricCollectionTimestamp
    
    elapsed = max(currentTs - lastMetricCollectionTimestamp, 0)
    requestRate = (requestCount - lastRequestCount) / elapsed if elapsed > 0 else requestCount
    diffMessageSent = (currentNbMessageSent - lastTotalMsgsMetric)
    nbMessageSentPerSec = diffMessageSent / elapsed if elapsed > 0 else currentNbMessageSent
    recordsPerRequestAvg = round(nbMessageSentPerSec / requestRate if nbMessageSentPerSec > 0 else 0, 2)

    lastMetricCollectionTimestamp = currentTs
    lastTotalMsgsMetric = currentNbMessageSent
    lastRequestCount = requestCount
    logger.info("Sent rate = %s/sec, duration spent in queue = %sms, batch size = %s, request rate = %s/sec, request latency avg = %sms, records per ProduceRequest = %s", nbMessageSentPerSec, queueTimeAvg, batchSizeAvg, requestRate, requestLatencyAvg, recordsPerRequestAvg);

def average(l):
    return sum(l) / len(l)

class Producer:
    def __init__(self, configs):
        self._producer = confluent_kafka.Producer(configs)
        self._cancelled = False
        #self._poll_thread = Thread(target=self._poll_loop)
        #self._poll_thread.start()

    def _poll_loop(self):
        while not self._cancelled:
            self._producer.poll(0.1)

    def close(self):
        self._cancelled = True
        #self._poll_thread.join()
    
    def produce2(self, topic, key, value):
        self._producer.produce(topic, key, value)
        #self._producer.poll(0)

    def produce(self, topic, key, value):
        while True:
            try:
                self._producer.produce(topic, key, value)
            except confluent_kafka.KafkaException as e:
                logger.error("Produce message failed: %s", str(e))
            except BufferError:
                logger.debug("Produce message queue full, waiting for deliveries")
                self._producer.poll(0.5)
                continue
            break
        self._producer.poll(0)

    def flush(self):
        self._producer.flush()

# ENTRYPOINT
if __name__ == '__main__':
    main()