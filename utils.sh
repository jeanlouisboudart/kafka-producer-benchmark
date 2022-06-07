#!/bin/bash

NETWORK_NAME=benchmark-producer-network
PRODUCER_IMAGES=("java-producer" "python-producer" "dotnet-producer" "rust-producer-sync")

docker_compose_file=${docker_compose_file:-docker-compose-3-brokers.yml}

build_image() {
  folder=$(basename $1)
  imageName=${2:-$folder}
  echo "Building docker image for $imageName"
  docker build -t $imageName ${folder}
}

build_all_images() {
  build_image bench-initializer 
  for producer_image in ${PRODUCER_IMAGES[@]}
  do
    build_image ${producer_image}
  done
}

create_bench_network() {
  echo "Creating network for the bench"
  docker network create ${NETWORK_NAME} || true
}

init_docker_compose_bench_env() {
  create_bench_network
  echo "Starting up "
  docker-compose -f ${docker_compose_file} up -d
  echo "Waiting for kafka to be up"
  sleep 10
}

stop_bench_network() {
  echo "Destroying network"
  docker network rm ${NETWORK_NAME}
}

stop_bench() {
  echo "Destroying environment"
  docker-compose -f ${docker_compose_file} down -v
  stop_bench_network
}

run_container() {
  imageName=$1
  envFile=$2
  docker run --env-file ${envFile} --rm --network ${NETWORK_NAME} ${imageName}
}

run_bench_initializer() {
  envFile=$1
  run_container bench-initializer ${envFile}
}

run_scenario_with_results() {
  imageName=$1
  scenario=$2
  echo "Running ${scenario} with ${imageName}"
  resultsFile=results/${scenario}/${imageName}.txt
  mkdir -p $(dirname ${resultsFile})
  run_container ${imageName} ${scenario} > $resultsFile 2>&1 || echo "Start of container ${imageName} failed check output logs !"
  
  echo "Results of the scenario dumped in ${resultsFile}"
  grep "REPORT" ${resultsFile} || true
}

run_scenario() {
  scenario=$1
  echo "Executing ${scenario} with the following characteristics"
  cat ${scenario}
  run_scenario_with_results bench-initializer ${scenario}
  for producer_image in ${PRODUCER_IMAGES[@]}
  do
    run_scenario_with_results ${producer_image} ${scenario}
  done
  
}

# terraform specifics
verify_terraform_installed() {
  if ! [ -x "$(command -v terraform)" ]; then
    echo "Terraform is not installed. Please install it, and then run this sript again."
    exit 1
  fi
  echo "Using terraform"
  terraform -version
}

unload_tfvars() {
  unset $(compgen -v TF_VAR)
}

load_scenario_in_tfvars() {
  unload_tfvars
  scenario=$1
  export $(cat ${scenario} | grep -v '^#' | grep -v "BOOTSTRAP_SERVER" |sed -E 's/(.*)/TF_VAR_\1/')
}

init_cloud_terraform() {
  terraform -chdir=cloud/setup init
}

run_scenario_cloud_terraform() {
  scenario=$1
  echo "Executing ${scenario} with the following characteristics"
  cat ${scenario}
  load_scenario_in_tfvars ${scenario}
  terraform -chdir=cloud/setup plan
  terraform -chdir=cloud/setup apply
  unload_tfvars
}

stop_bench_cloud_terraform() {
  terraform -chdir=cloud/setup destroy
}