#!/bin/bash


SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
SCRIPTDIR=$(basename $SCRIPTPATH)

source utils.sh

build_all_images
init_docker_compose_bench_env

for scenario in $(ls scenario-*.env)
do
  run_scenario ${scenario}
done

read -p "Press enter to continue"

stop_bench