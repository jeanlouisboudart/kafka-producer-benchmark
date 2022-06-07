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

variable "NB_TOPICS" {
  description = "Number of topics to create"
  type        = number
  default     = 100
}

variable "NUMBER_OF_PARTITIONS" {
  description = "Number of partitions for each topic created"
  type        = number
  default     = 6
}

variable "TOPIC_PREFIX" {
  description = "Prefix for topic names. Topic name will be: {topic_prefix}_{i}"
  type        = string
  default     = "sample"
}

variable "MESSAGE_SIZE" {
  description = "Message size in bytes"
  type        = number
  default     = 200
}

variable "NB_MESSAGES" {
  description = "Number of messages to inject"
  type        = number
  default     = 1000000
}

variable "USE_RANDOM_KEYS" {
  description = "Use message keys. If false, it will do round robin."
  type        = boolean
  default     = true
}

variable "USE_RANDOM_KEYS" {
  description = "Use message keys. If false, it will do round robin."
  type        = boolean
  default     = true
}

variable "AGG_PER_TOPIC_NB_MESSAGES" {
  description = "Aggregate messages per topic. Value of 1 means no aggregation."
  type        = number
  default     = 1000
}

variable "KAFKA_BATCH_SIZE" {
  description = "The producer will attempt to batch records together into fewer requests whenever multiple records are being sent to the same partition. This helps performance on both the client and the server. This configuration controls the default batch size in bytes."
  type        = number
  default     = 100000
}

variable "KAFKA_LINGER_MS" {
  description = "The producer groups together any records that arrive in between request transmissions into a single batched request. Normally this occurs only under load when records arrive faster than they can be sent out. However in some circumstances the client may want to reduce the number of requests even under moderate load. This setting accomplishes this by adding a small amount of artificial delay—that is, rather than immediately sending out a record, the producer will wait for up to the given delay to allow other records to be sent so that the sends can be batched together. This can be thought of as analogous to Nagle's algorithm in TCP. This setting gives the upper bound on the delay for batching: once we get batch.size worth of records for a partition it will be sent immediately regardless of this setting, however if we have fewer than this many bytes accumulated for this partition we will 'linger' for the specified time waiting for more records to show up. This setting defaults to 0 (i.e. no delay). Setting linger.ms=5, for example, would have the effect of reducing the number of requests sent but would add up to 5ms of latency to records sent in the absence of load."
  type        = number
  default     = 10
}

variable "KAFKA_MAX_IN_FLIGHT_REQUESTS_PER_CONNECTION" {
  description = "The maximum number of unacknowledged requests the client will send on a single connection before blocking. Note that if this config is set to be greater than 1 and enable.idempotence is set to false, there is a risk of message re-ordering after a failed send due to retries (i.e., if retries are enabled)."
  type        = number
  default     = 5
}

variable "k8s_context" {
  description = "The k8s context to connect to. Null will use the default k8s context on your machine."
  type        = string
  default     = null
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
