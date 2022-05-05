#!/bin/bash

NETWORK_NAME=benchmark-producer-network
PRODUCER_IMAGES=("java-producer" "python-producer")

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

create_network() {
  echo "Creating network for the bench"
  docker network create ${NETWORK_NAME}
}

init_docker_compose_bench_env() {
  docker_compose_file=${1:-docker-compose-3-brokers.yml} 
  create_network
  echo "Starting up "
  docker-compose -f ${docker_compose_file} up -d
  echo "Waiting for kafka to be up"
  sleep 10
}

stop_bench() {
  echo "Destroying environment"
  docker-compose -f ${docker_compose_file} down -v
  echo "Destroying network"
  docker network rm ${NETWORK_NAME}
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
  run_container $1 $2 > $resultsFile 2>&1
  echo "Results of the scenario dumped in ${resultsFile}"
  grep "REPORT" ${resultsFile}
}

run_scenario() {
  scenario=$1
  run_bench_initializer ${scenario}
  for producer_image in ${PRODUCER_IMAGES[@]}
  do
    run_scenario_with_results ${producer_image} ${scenario}
  done
  
}