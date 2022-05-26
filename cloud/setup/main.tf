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

// 'app-manager' service account is required in this configuration to create 'orders' topic and grant ACLs
// to 'app-producer' and 'app-consumer' service accounts.
resource "confluent_service_account" "app-manager" {
  display_name = "app-manager"
  description  = "Service account to manage 'Producer Perf Test' Kafka cluster"
}

resource "confluent_role_binding" "app-manager-kafka-cluster-admin" {
  principal   = "User:${confluent_service_account.app-manager.id}"
  role_name   = "CloudClusterAdmin"
  crn_pattern = confluent_kafka_cluster.perf-test-dedicated.rbac_crn
}

resource "confluent_api_key" "app-manager-kafka-api-key" {
  display_name = "app-manager-kafka-api-key"
  description  = "Kafka API Key that is owned by 'app-manager' service account"
  owner {
    id          = confluent_service_account.app-manager.id
    api_version = confluent_service_account.app-manager.api_version
    kind        = confluent_service_account.app-manager.kind
  }

  managed_resource {
    id          = confluent_kafka_cluster.perf-test-dedicated.id
    api_version = confluent_kafka_cluster.perf-test-dedicated.api_version
    kind        = confluent_kafka_cluster.perf-test-dedicated.kind

    environment {
      id = var.confluent_environment_id
    }
  }

  # The goal is to ensure that confluent_role_binding.app-manager-kafka-cluster-admin is created before
  # confluent_api_key.app-manager-kafka-api-key is used to create instances of
  # confluent_kafka_topic, confluent_kafka_acl resources.

  # 'depends_on' meta-argument is specified in confluent_api_key.app-manager-kafka-api-key to avoid having
  # multiple copies of this definition in the configuration which would happen if we specify it in
  # confluent_kafka_topic, confluent_kafka_acl resources instead.
  depends_on = [
    confluent_role_binding.app-manager-kafka-cluster-admin
  ]
}

resource "confluent_service_account" "perf-test-client" {
  display_name = "perf-test-client"
  description  = "Service account to run performance tests"
}

resource "confluent_kafka_acl" "perf-client-can-write-to-topics" {
  kafka_cluster {
    id = confluent_kafka_cluster.perf-test-dedicated.id
  }
  resource_type = "TOPIC"
  resource_name = "ANY"
  pattern_type  = "LITERAL"
  principal     = "User:${confluent_service_account.perf-test-client.id}"
  host          = "*"
  operation     = "WRITE"
  permission    = "ALLOW"
  http_endpoint = confluent_kafka_cluster.perf-test-dedicated.http_endpoint
  credentials {
    key    = confluent_api_key.app-manager-kafka-api-key.id
    secret = confluent_api_key.app-manager-kafka-api-key.secret
  }
}

resource "confluent_kafka_acl" "perf-client-can-read-from-topics" {
  kafka_cluster {
    id = confluent_kafka_cluster.perf-test-dedicated.id
  }
  resource_type = "TOPIC"
  resource_name = "ANY"
  pattern_type  = "LITERAL"
  principal     = "User:${confluent_service_account.perf-test-client.id}"
  host          = "*"
  operation     = "READ"
  permission    = "ALLOW"
  http_endpoint = confluent_kafka_cluster.perf-test-dedicated.http_endpoint
  credentials {
    key    = confluent_api_key.app-manager-kafka-api-key.id
    secret = confluent_api_key.app-manager-kafka-api-key.secret
  }
}

resource "confluent_kafka_acl" "perf-client-can-list-groups" {
  kafka_cluster {
    id = confluent_kafka_cluster.perf-test-dedicated.id
  }
  resource_type = "GROUP"
  resource_name = "ANY"
  pattern_type  = "LITERAL"
  principal     = "User:${confluent_service_account.perf-test-client.id}"
  host          = "*"
  operation     = "READ"
  permission    = "ALLOW"
  http_endpoint = confluent_kafka_cluster.perf-test-dedicated.http_endpoint
  credentials {
    key    = confluent_api_key.app-manager-kafka-api-key.id
    secret = confluent_api_key.app-manager-kafka-api-key.secret
  }
}

resource "confluent_api_key" "perf-test-client-api-key" {
  display_name = "perf-test-client-api-key"
  description  = "API Key owned by 'perf-test-client' service account to produce/consume to/from Kafka topics"
  owner {
    id          = confluent_service_account.perf-test-client.id
    api_version = confluent_service_account.perf-test-client.api_version
    kind        = confluent_service_account.perf-test-client.kind
  }

  managed_resource {
    id          = confluent_kafka_cluster.perf-test-dedicated.id
    api_version = confluent_kafka_cluster.perf-test-dedicated.api_version
    kind        = confluent_kafka_cluster.perf-test-dedicated.kind

    environment {
      id = var.confluent_environment_id
    }
  }
}
