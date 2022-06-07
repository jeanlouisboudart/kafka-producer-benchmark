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

for scenario in $(ls scenario-*.env)
do
    run_scenario_cloud_terraform $scenario
done

stop_bench_cloud_terraform