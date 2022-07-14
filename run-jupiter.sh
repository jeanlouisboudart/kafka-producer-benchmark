#!/bin/bash

set -e

SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
SCRIPTDIR=$(basename $SCRIPTPATH)

source utils.sh

cd ${BENCH_ROOT_DIR}/report
docker-compose up -d --build

echo "Jupiter is available http://localhost:8888"
echo "You can load the notebookd in work/{SCENARIO}/benchmark-report.ipynb"

read -p "Press enter to stop jupiter"
docker-compose down -v
