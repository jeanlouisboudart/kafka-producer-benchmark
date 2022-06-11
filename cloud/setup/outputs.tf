output "resource-ids" {
  value = <<-EOT
  Environment ID:   ${var.confluent_environment_id}
  Kafka Cluster ID: ${confluent_kafka_cluster.perf-test-dedicated.id}
  Service Accounts and their Kafka API Keys (API Keys inherit the permissions granted to the owner):
  ${confluent_service_account.app-manager.display_name}:                     ${confluent_service_account.app-manager.id}
  ${confluent_service_account.app-manager.display_name}'s Kafka API Key:     "${confluent_api_key.app-manager-kafka-api-key.id}"
  ${confluent_service_account.app-manager.display_name}'s Kafka API Secret:  "${confluent_api_key.app-manager-kafka-api-key.secret}"
  ${confluent_service_account.perf-test-client.display_name}:                    ${confluent_service_account.perf-test-client.id}
  ${confluent_service_account.perf-test-client.display_name}'s Kafka API Key:    "${confluent_api_key.perf-test-client-api-key.id}"
  ${confluent_service_account.perf-test-client.display_name}'s Kafka API Secret: "${confluent_api_key.perf-test-client-api-key.secret}"

  In order to use the Confluent CLI v2 to produce and consume messages from topics using Kafka API Keys
  of ${confluent_service_account.perf-test-client.display_name} service account:
  run the following commands:
  # 1. Log in to Confluent Cloud
  $ confluent login
  # 2. Produce key-value records to topic XXX by using ${confluent_service_account.perf-test-client.display_name}'s Kafka API Key
  $ confluent kafka topic produce XXX --environment ${var.confluent_environment_id} --cluster ${confluent_kafka_cluster.perf-test-dedicated.id} --api-key "${confluent_api_key.perf-test-client-api-key.id}" --api-secret "${confluent_api_key.perf-test-client-api-key.secret}"
  # Enter a few records and then press 'Ctrl-C' when you're done.
  EOT
  sensitive = true
}