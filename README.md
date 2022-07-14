# Kafka Producer Benchmark

Simple benchmark to compare different clients performance against similar configuration.

The project is relatively low tech and just dumping similar metrics against different client into files.

# Which metrics are captured ?

The following metrics are collected (average across all topic partitions):
* Send rate = Number of messages sent per seconds
* Duration spent in the local queue = How many ms messages stayed in the queue before being sent ?
* Batch size = Average batch size
* Request Rate = number of Produce Request per seconds
* Request Latency = Average latency of Produce Request
* Records per Produce Request = Average number of records per Produce Request

At the end of the test all the messages are flushed then the benchmark will display the number of messages sent, the duration and number of produce request made.

# How to run it ?
## Running all scenarios
To run all scenarios use the following command :
```
./run-scenarios.sh
```
The script will execute all the scenario files named `scenario-<description>.env` at the root of the project against each Kafka Producer clients registered in the project.

Execution logs will be dumped in `./target/scenario-<description>.env/<client-name>.txt`.

## Running a single scenario

To run a scenario use the following command :
```
./run-scenario.sh <myscenario>.env
```

Execution logs will be dumped in `./target/scenario-<description>.env/<client-name>.txt`.

## Running with a custom `docker-compose` file

Just set `docker_compose_file=...` variable in the shell you're running `run-scenario` from.
For instance: 
```
docker_compose_file=docker-compose-kraft-3-brokers.yml;./run-scenario.sh <myscenario>.env
```

# How to run the scenarios on a Confluent Cloud cluster ?
The repository now contains scripts to run the producer benchmark on Kubernetes connected to a Confluent Cloud cluster.

## Pre-requisites 
Pre-requisites are :
* Terraform installed
* Having a kubernetes environment to run the producer benchmark 
* Making sure you have a current context (kubectl config get-contexts)

## Configuring the parameters for Confluent Cloud
Create a file called `secrets.auto.tfvars` in the `cloud/setup` folder with the following content :
```
confluent_org_id            = "<YOUR_CCLOUD_ORG_ID>"
confluent_environment_id    = "<YOUR_CCLOUD_ENV_ID>" 
confluent_cloud_api_key = "<YOUR_CCLOUD_API_KEY>"
confluent_cloud_api_secret = "<YOUR_CCLOUD_API_KEY>"
```

To know all the variables you can tweak, please read [variable file](cloud/setup/variables.tf)

## Running all scenarios
To run all scenarios use the following command :
```
./run-scenarios-cloud.sh
```
The script will execute all the scenario files named `scenario-<description>.env` at the root of the project against each Kafka Producer clients registered in the project.

## Running a single scenario

To run a scenario use the following command :
```
./run-scenario-cloud.sh <myscenario>.env
```

# How to contribute ?
Have any idea to make this benchmark better ? Found a bug ?
Do not hesitate to report us via github issues and/or create a pull request.

## Adding a new scenario ?

The `./run-scenarios.sh` script is looking for all files matching the patern `scenario-<description>.env`.
Existing scenarios has been named with the following naming convention `scenario-<nbtopics>t<nbpartitions>p-<description>.env`.

The easiest way to create a new scenario would be to duplicate an existing scenario file and play with the values.
You can override any producer configuration available in the clients by using the following naming conventions :
* Prefix with KAFKA_.
* Convert to upper-case.
* Replace a period (.) with a single underscore (_).


## Adding a new client ?
We strongly recommend you to run your test against localhost:9092 you can leverage the default `docker-compose.yml` to have a development environment.
If interested in experimenting with KIP-500, you can run your implementation locally against `docker compose -f docker-compose-kraft.yml` (1 single node acting as both controller and broker).

Make everything configurable via environment variable.

Default variables are :
* KAFKA_BOOTSTRAP_SERVERS=localhost
* NB_TOPICS=1
* REPLICATION_FACTOR=1
* NUMBER_OF_PARTITIONS=6
* MESSAGE_SIZE=200
* NB_MESSAGES=1000000
* REPORTING_INTERVAL=1000
* USE_RANDOM_KEYS=true
* AGG_PER_TOPIC_NB_MESSAGES=1
Those variables should be used by all clients to makes things easier to configure, but each client implementation can have its own set of custom configuration variables.

Convert KAFKA_XXX into lowercase by replacing "_" with dots.
This will help to play with batch.size/linger.ms/etc...

### Specs 
By default each client implementation will need to capture metrics at regular interval (defined via REPORTING_INTERVAL).
The should be logged using the following format to make things easier to compare: 
```java
        logger.info("Sent rate = {}/sec, duration spent in queue = {}ms, batch size = {}, request rate = {}/sec, request latency avg = {}ms, records per ProduceRequest = {}", avgSendRate, queueTimeAvg, batchSizeAvg, requestRate, requestLatencyAvg, recordsPerRequestAvg);

```

At the end of the run make sure all messages are delivered (ex. by calling producer.flush().

At the end of the run make sure you produce a log starting with "REPORT" keyword, this will be displayed at when executing scenarios.
Example:
```java
    logger.info("REPORT: Produced %s with %s ProduceRequests in %s", lastTotalMsgsMetric, lastRequestCount, str(timedelta(seconds=end_time - start_time)))
```

### My client is ready, how can I plug it in the test suite ?
Create a new folder at the root of the project
Make sure you have a `Dockerfile` inside of it.

Update the `PRODUCER_IMAGES` in `utils.sh` to reference your new client.
This will be taken into account to build the image but also to start the scenarios.