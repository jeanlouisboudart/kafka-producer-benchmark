// Example function-based Apache Kafka producer
package main

import (
	"benchmark/producer/config"
	"benchmark/producer/stats"
	"benchmark/producer/utils"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"time"

	"github.com/confluentinc/confluent-kafka-go/kafka"
	"github.com/google/uuid"
)

type Message struct {
	Key   string
	Value string
}

func main() {

	// Load configuration
	config, err := config.LoadConfig("./.env")
	if err != nil {
		log.Fatal("cannot load config:", err)
	}
	log.Println(config)

	producerConfig := &kafka.ConfigMap{} // create config map for producer
	for k, v := range config.KAFKA_CONFIGS {
		producerConfig.SetKey(k, v)
	}

	// Create a new producer instance
	totalMsgcnt := config.NB_MESSAGES

	p, err := kafka.NewProducer(producerConfig)

	if err != nil {
		fmt.Printf("Failed to create producer: %s\n", err)
		os.Exit(1)
	}

	fmt.Printf("Created Producer %v\n", p)

	var statsMetrics stats.LastMetrics

	// Listen to all the events on the default events channel
	go func() {
		for e := range p.Events() {
			switch ev := e.(type) {
			case *kafka.Message:
				m := ev
				if m.TopicPartition.Error != nil {
					fmt.Printf("Delivery failed: %v\n", m.TopicPartition.Error)
				} else {
					/* fmt.Printf("Delivered message to topic %s [%d] at offset %v\n",
					*m.TopicPartition.Topic, m.TopicPartition.Partition, m.TopicPartition.Offset) */
				}
			case kafka.Error:
				fmt.Printf("Error: %v\n", ev)
			case *kafka.Stats: // stats event
				var statsResult stats.Stats
				json.Unmarshal([]byte(e.String()), &statsResult)
				statsMetrics = stats.ProcessStats(statsResult)
			default:
				fmt.Printf("Ignored event: %s\n", ev)
			}
		}
	}()

	log.Printf("Running benchmark with %d topics %d messages of %d bytes each with random keys=%t", config.NB_TOPICS, config.NB_MESSAGES, config.MESSAGE_SIZE, config.USE_RANDOM_KEYS)
	if config.AGG_PER_TOPIC_NB_MESSAGES > 1 {
		log.Printf("Will use grouping per topic and bulk send every %d messages", config.AGG_PER_TOPIC_NB_MESSAGES)
	}

	// Fake data
	nbFakeData := config.NB_TOPICS * 1000

	//Values
	values := make([]string, nbFakeData)
	for i := 0; i < nbFakeData; i++ {
		values[i] = utils.RandStringBytesMaskImprSrcUnsafe(200)
	}

	// Generate keys
	uuids := make([]string, nbFakeData)
	for i := 0; i < nbFakeData; i++ {
		uuids[i] = uuid.New().String()
	}

	msgcnt := 0
	toSend := make(map[string][]Message)
	start := time.Now()

	for msgcnt < totalMsgcnt {
		topic := config.TOPIC_PREFIX + "_" + strconv.Itoa(msgcnt%config.NB_TOPICS)
		index := totalMsgcnt % nbFakeData
		var key string
		if config.USE_RANDOM_KEYS {
			key = uuids[index] // random key
		}
		value := values[index]

		if config.AGG_PER_TOPIC_NB_MESSAGES > 1 {
			toSend[topic] = append(toSend[topic], Message{Key: key, Value: value})
			if totalMsgcnt%config.AGG_PER_TOPIC_NB_MESSAGES == 0 {
				for topic, messages := range toSend {
					for _, message := range messages {
						err = p.Produce(&kafka.Message{
							TopicPartition: kafka.TopicPartition{Topic: &topic, Partition: kafka.PartitionAny},
							Key:            []byte(message.Key),
							Value:          []byte(message.Value),
							Headers:        nil,
						}, nil)

						if err != nil {
							if err.(kafka.Error).Code() == kafka.ErrQueueFull {
								// Producer queue is full, wait 1s for messages
								// to be delivered then try again.
								time.Sleep(time.Second)
								continue
							}
							fmt.Printf("Failed to produce message: %v\n", err)
						}
					}
				}
				toSend = make(map[string][]Message)
			}
		} else {
			err = p.Produce(&kafka.Message{
				TopicPartition: kafka.TopicPartition{Topic: &topic, Partition: kafka.PartitionAny},
				Key:            []byte(key),
				Value:          []byte(value),
				Headers:        nil,
			}, nil)

			if err != nil {
				if err.(kafka.Error).Code() == kafka.ErrQueueFull {
					// Producer queue is full, wait 1s for messages
					// to be delivered then try again.
					time.Sleep(time.Second)
					continue
				}
				fmt.Printf("Failed to produce message: %v\n", err)
			}
		}
		msgcnt++
	}

	// Flush and close the producer and the events channel
	for p.Flush(10000) > 0 {
		fmt.Print("Still waiting to flush outstanding messages\n", err)
	}
	elapsed := time.Since(start)

	p.Close()
	log.Printf("REPORT: Produced %d with %d ProduceRequest in %s", statsMetrics.LastTotalMsgsMetric, statsMetrics.LastRequestCount, elapsed)
}
