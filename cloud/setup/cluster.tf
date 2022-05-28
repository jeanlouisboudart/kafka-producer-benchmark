resource "confluent_kafka_cluster" "perf-test-dedicated" {
  display_name = "Producer Perf Test"
  availability = "SINGLE_ZONE"
  cloud        = "GCP"
  region       = "europe-west1"
  dedicated {
    cku = 1
  }
  environment {
    id = var.confluent_environment_id
  }
}

resource "confluent_kafka_topic" "topics" {
  count = var.nb_topics
  kafka_cluster {
    id = confluent_kafka_cluster.perf-test-dedicated.id
  }
  topic_name         = "${var.topic_prefix}_${count.index}"
  partitions_count   = var.nb_partitions
  http_endpoint      = confluent_kafka_cluster.perf-test-dedicated.http_endpoint
  config = {
    "cleanup.policy"    = "delete"
    "retention.ms"      = "604800000" # 7 days
  }
  credentials {
    key    = confluent_api_key.app-manager-kafka-api-key.id
    secret = confluent_api_key.app-manager-kafka-api-key.secret
  }
}