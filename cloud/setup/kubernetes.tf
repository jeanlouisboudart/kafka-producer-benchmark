resource "kubernetes_namespace" "kafka-producer-bench-ns" {
  metadata {
    name = "kafka-producer-benchmark"
  }
}

resource "kubernetes_job" "producer-benchmark" {
  depends_on = [confluent_kafka_topic.topics]
  metadata {
    name = "producer-benchmark"
    namespace = kubernetes_namespace.kafka-producer-bench-ns.id
  }
  spec {
    parallelism = var.producer_instances
    template {
      metadata {}
      spec {
        container {
          name  = "producer-benchmark"
          image = var.bench_producer_image
          image_pull_policy = var.image_pull_policy
          env {
            name = "KAFKA_BOOTSTRAP_SERVERS"
            value = confluent_kafka_cluster.perf-test-dedicated.bootstrap_endpoint
            #value = "localhost:9092"
          }
          env {
            name = "KAFKA_SECURITY_PROTOCOL"
            value = "SASL_SSL"
          }
          env {
            name = "KAFKA_SASL_MECHANISMS"
            value = "PLAIN"
          }

          env {
            name = "KAFKA_SASL_MECHANISM"
            value = "PLAIN"
          }
          env {
            name = "KAFKA_SASL_MECHANISMS"
            value = "PLAIN"
          }
          # Java specifics
          env {
            name  = "KAFKA_SASL_JAAS_CONFIG"
            value = "org.apache.kafka.common.security.plain.PlainLoginModule   required username='${confluent_api_key.perf-test-client-api-key.id}'   password='${confluent_api_key.perf-test-client-api-key.secret}';"
          }
#          librdkafka specifics
          env {
            name = "KAFKA_SASL_USERNAME"
            value = confluent_api_key.perf-test-client-api-key.id
          }
          env {
            name = "KAFKA_SASL_PASSWORD"
            value = confluent_api_key.perf-test-client-api-key.secret
          }
          # Batching configuration
          env { # GroupBy(key)
            name = "AGG_PER_TOPIC_NB_MESSAGES"
            value = "1000"
          }

          env {
            name = "KAFKA_BATCH_SIZE"
            value = "100000"
          }
          env {
            name = "KAFKA_LINGER_MS"
            value = "10"
          }
          env {
            name = "KAFKA_MAX_IN_FLIGHT_REQUESTS_PER_CONNECTION"
            value = "5"
          }
          env {
            name = "NB_TOPICS"
            value = var.nb_topics
          }
          env {
            name = "NUMBER_OF_PARTITIONS"
            value = var.nb_partitions
          }
          env {
            name = "MESSAGE_SIZE"
            value = "200"
          }
          env {
            name = "NB_MESSAGES"
            value = "1000000"
          }
          env {
            name = "USE_RANDOM_KEYS"
            value = "true"
          }
#          env {
#            name = "RUST_LOG"
#            value = "DEBUG,librdkafka=trace,rdkafka::client=debug"
#          }
        }
        restart_policy = "Never"
      }
    }
    backoff_limit = 4
    completions = var.producer_instances
  }
  wait_for_completion = true
  timeouts {
    create = "5m"
    update = "5m"
  }
}