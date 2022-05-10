#!/bin/bash

set -e

SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
SCRIPTDIR=$(basename $SCRIPTPATH)

source utils.sh

if [ $# -lt 1 ]; then
    echo "Usage $0 <scenario-name>"  
    exit 1
fi

build_all_images
init_docker_compose_bench_env

scenario=$1

run_scenario $scenario

read -p "Press enter to continue"

stop_bench