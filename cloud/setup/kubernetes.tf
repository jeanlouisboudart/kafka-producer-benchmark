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
          # Loading variables from scenario .env file
          dynamic "env" {
            for_each = data.external.load_scenario_file.result
            content {
              name = env.key
              value = env.value
            }
          }
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