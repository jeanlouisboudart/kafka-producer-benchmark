resource "kubernetes_namespace" "kafka-producer-bench-ns" {
  metadata {
    name = "kafka-producer-benchmark"
  }
}

#---
#apiVersion: v1
#kind: PersistentVolumeClaim
#metadata:
#name: pvc-name
#spec:
#accessModes:
#- ReadWriteMany
#resources:
#requests:
#storage: 1Gi

resource "kubernetes_persistent_volume_claim" "report-pvc" {
  metadata {
    name = "report-pvc"
  }
  spec {
    access_modes = [
      "ReadWriteMany"
    ]
    resources {
      requests = {
        storage = "256Mi"
      }
    }
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
          volume_mount {
            mount_path = "results"
            name       = "report"
          }
        }
        volume {
          name = "report"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.report-pvc.metadata.0.name
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