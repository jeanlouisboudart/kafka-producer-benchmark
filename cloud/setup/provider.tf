terraform {
  required_providers {
    confluent = {
      source  = "confluentinc/confluent"
      version = "0.8.1"
    }
  }
}

provider "confluent" {
  api_key    = var.confluent_cloud_api_key
  api_secret = var.confluent_cloud_api_secret
}