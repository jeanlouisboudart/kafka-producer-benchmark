#!/bin/bash

set -e

SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
SCRIPTDIR=$(basename $SCRIPTPATH)

source utils.sh

if [ $# -lt 1 ]; then
    echo "Usage $0 <scenario-name>"  
    exit 1
fi

verify_terraform_installed
build_all_images
init_cloud_terraform

scenario=$1

run_scenario_cloud_terraform ${SCRIPTPATH} ${scenario}

stop_bench_cloud_terraform