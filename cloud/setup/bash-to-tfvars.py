#!/usr/bin/env python3
import re
import csv
import json
import sys
import os.path

# parse terraform inputs
# see  https://registry.terraform.io/providers/hashicorp/external/latest/docs/data-sources/data_source
input = sys.stdin.read()
input_json = json.loads(input)
filename=input_json['filename']

# parse the file requested as parameter 
lines = []
with open(filename) as stream:
    contents = stream.read().strip()
var_declarations = re.findall(r"^[a-zA-Z0-9_]+=.*$", contents, flags=re.MULTILINE)
# FIXME: this will remove the KAFKA_BOOTSTRAP_SERVERS as it's already handled at terraform level
# We might get rid of this if we refactor a bit how scenarios files are constructred in future to get all paramerters
# related to performance and no connection parameters (bootstrap, security, etc...)
var_without_bootstrap = [line for line in var_declarations if not line.startswith("KAFKA_BOOTSTRAP_SERVERS")]
reader = csv.reader(var_without_bootstrap, delimiter="=")
bash_vars = dict(reader)

# output the results in json format to match terraform API
asjson = json.dumps(bash_vars,indent=2)
print(asjson)