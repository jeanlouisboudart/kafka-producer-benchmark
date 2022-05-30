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

variable "k8s_context" {
  description = "The k8s context to connect to"
  type        = string
  default     = "docker-desktop"
}

variable "producer_instances" {
  description = "How many producer instances to run in parallel"
  type        = number
  default     = 10
}

variable "bench_producer_image" {
  description = "Docker image FQ name to benchmark"
  type        = string
}

variable "image_pull_policy" {
  description = "Image pull policy for producer benchmark docker images, use Never when using docker desktop"
  type        = string
  default     = "Never"
}
