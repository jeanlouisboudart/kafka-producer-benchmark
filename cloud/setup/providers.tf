terraform {
  required_providers {
    confluent = {
      source  = "confluentinc/confluent"
      version = "0.8.1" # 0.9 has a bug with timeouts and RBAC at the moment
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.11"
    }
  }
}

provider "confluent" {
  api_key    = var.confluent_cloud_api_key
  api_secret = var.confluent_cloud_api_secret
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = var.k8s_context
}