#!/bin/bash

NETWORK_NAME=benchmark-producer-network
PRODUCER_IMAGES=("java-producer" "python-producer" "dotnet-producer" "rust-producer-sync" "golang-producer")
BENCH_ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

docker_compose_file=${docker_compose_file:-docker-compose-3-brokers.yml}

build_image() {
  folder=$(basename $1)
  imageName=${2:-$folder}
  echo "Building docker image for $imageName"
  docker build -t $imageName ${folder}
}

build_jupiter_image() {
  # current uid and gid
  curr_uid=`id -u`
  curr_gid=`id -g`
  echo $BENCH_ROOT_DIR
  docker build --build-arg UID=${curr_uid} --build-arg GID=${curr_gid} -f ${BENCH_ROOT_DIR}/report/Dockerfile -t jupyter-converter ${BENCH_ROOT_DIR}/report
}

build_all_images() {
  build_image bench-initializer 
  for producer_image in ${PRODUCER_IMAGES[@]}
  do
    build_image ${producer_image}
  done
  build_jupiter_image 
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
  resultsFolder=$3
  echo "Running ${scenario} with ${imageName}"
  resultsFile="${resultsFolder}/${imageName}.txt"
  mkdir -p $(dirname ${resultsFile})
  run_container ${imageName} ${scenario} > $resultsFile 2>&1 || echo "Start of container ${imageName} failed check output logs !"
  
  echo "Results of the scenario dumped in ${resultsFile}"
  grep "REPORT" ${resultsFile} || true
}

run_scenario() {
  scenario=$1
  echo "Executing ${scenario} with the following characteristics"
  cat ${scenario}
  resultsFolder="${BENCH_ROOT_DIR}/results/${scenario}"
  run_scenario_with_results bench-initializer ${scenario} ${resultsFolder}
  for producer_image in ${PRODUCER_IMAGES[@]}
  do
    run_scenario_with_results ${producer_image} ${scenario} ${resultsFolder}
  done
  generate_reports ${scenario} ${resultsFolder}
}

generate_reports() {
  scenario=$1
  resultsFolder=$2
  cp ${scenario} ${resultsFolder}/scenario.env
  cp ${BENCH_ROOT_DIR}/report/benchmark-report.ipynb ${resultsFolder}/benchmark-report.ipynb

  ${BENCH_ROOT_DIR}/report/extract_results.py ${resultsFolder}
  docker run --rm -v ${resultsFolder}:/home/jovyan/work jupyter-converter
  echo "Markdown report generated in ${resultsFolder}/report.md"
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

init_cloud_terraform() {
  terraform -chdir=cloud/setup init
}

run_scenario_cloud_terraform() {
  scenario_folder=$1
  scenario=$2
  echo "Executing ${scenario} with the following characteristics"
  cat ${scenario}
  for producer_image in ${PRODUCER_IMAGES[@]}
  do
    echo "Executing"
    terraform -chdir=cloud/setup plan -var "scenario_file=${scenario_folder}/${scenario}" -var "bench_producer_image=${producer_image}"
    terraform -chdir=cloud/setup apply -auto-approve
  done
}

stop_bench_cloud_terraform() {
  terraform -chdir=cloud/setup destroy
}