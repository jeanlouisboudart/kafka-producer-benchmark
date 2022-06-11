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
  default     = "java-producer:latest"
}

variable "image_pull_policy" {
  description = "Image pull policy for producer benchmark docker images, use Never when using docker desktop"
  type        = string
  default     = "Never"
}

# Load scenario file as terraform datasource 
# https://registry.terraform.io/providers/hashicorp/external/latest/docs/data-sources/data_source
variable "scenario_file" {
  description = "Path of the scenario variable files"
  type        = string
  default     = "../../scenario-1t6p-java-defaults.env"
}

# Load external
data "external" "load_scenario_file" {
  program = ["python3", "bash-to-tfvars.py"]
  query = {
    filename = var.scenario_file
  }
}

locals {
  NB_TOPICS = data.external.load_scenario_file.result.NB_TOPICS
  TOPIC_PREFIX = data.external.load_scenario_file.result.TOPIC_PREFIX
  NUMBER_OF_PARTITIONS = data.external.load_scenario_file.result.NUMBER_OF_PARTITIONS
}