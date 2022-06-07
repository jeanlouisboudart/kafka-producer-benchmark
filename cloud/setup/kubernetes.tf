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
          # librdkafka specifics
          env {
            name = "KAFKA_SASL_USERNAME"
            value = confluent_api_key.perf-test-client-api-key.id
          }
          env {
            name = "KAFKA_SASL_PASSWORD"
            value = confluent_api_key.perf-test-client-api-key.secret
          }
          # Batching configuration
          env {
            name = "KAFKA_BATCH_SIZE"
            value = var.KAFKA_BATCH_SIZE
          }
          env {
            name = "KAFKA_LINGER_MS"
            value = var.KAFKA_LINGER_MS
          }
          env {
            name = "KAFKA_MAX_IN_FLIGHT_REQUESTS_PER_CONNECTION"
            value = var.KAFKA_MAX_IN_FLIGHT_REQUESTS_PER_CONNECTION
          }
          env { 
            name = "AGG_PER_TOPIC_NB_MESSAGES"
            value = var.AGG_PER_TOPIC_NB_MESSAGES
          }
          env {
            name = "NB_TOPICS"
            value = var.NB_TOPICS
          }
          env {
            name = "NUMBER_OF_PARTITIONS"
            value = var.NUMBER_OF_PARTITIONS
          }
          env {
            name = "MESSAGE_SIZE"
            value = var.MESSAGE_SIZE
          }
          env {
            name = "NB_MESSAGES"
            value = var.NB_MESSAGES
          }
          env {
            name = "USE_RANDOM_KEYS"
            value = var.USE_RANDOM_KEYS
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