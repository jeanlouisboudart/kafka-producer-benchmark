variable "confluent_cloud_api_key" {
  description = "Confluent Cloud API Key (also referred as Cloud API ID)"
  type        = string
  sensitive   = true
}

variable "confluent_cloud_api_secret" {
  description = "Confluent Cloud API Secret"
  type        = string
  sensitive   = true
}

variable "confluent_environment_id" {
  description = "Confluent Environment ID"
  type        = string
}

variable "confluent_org_id" {
  description = "Confluent Organization ID"
  type        = string
}

variable "nb_topics" {
  description = "Number of topics to create"
  type        = number
  default     = 100
}

variable "nb_partitions" {
  description = "Number of partitions for each topic created"
  type        = number
  default     = 6
}

variable "topic_prefix" {
  description = "Prefix for topic names. Topic name will be: {topic_prefix}_{i}"
  type        = string
  default     = "sample"
}