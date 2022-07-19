#!/usr/bin/env python3
import re
import sys
from os import listdir
from os.path import isfile, join

extracted_metrics = [
    'Sent rate',
    'duration spent in queue',
    'batch size',
    'request rate',
    'request latency avg',
    'records per ProduceRequest'
]
metric_patterns = [fr"{metric}\s*=\s*(NaN|\d*\.?\d*)" for metric in extracted_metrics]
metrics_pattern = re.compile('.*' + '.*'.join(metric_patterns))
report_pattern = re.compile(r'.*REPORT.*\s*Produced\s*(\d+?\.?\d+)\s+with\s+(\d+?\.?\d+)\s+ProduceRequests\s+in\s+(.*)ms.*')
file_lang_pattern = re.compile('.*/(.*)-producer.*[.]txt')


def main(args):
    results_folder = args[0]
    metric_output_csv= join(results_folder, 'metrics.csv')
    with open(metric_output_csv, 'w') as f:
        print("ProducerType,Index,", end='', file=f)
        print(','.join(extracted_metrics), file=f)

    final_output_csv= join(results_folder, 'final-results.csv')
    with open(final_output_csv, 'w') as f:
        print("ProducerType,nb_msgs,nb_requests,duration",  file=f)


    for file in listdir(results_folder):
        fullpath = join(results_folder, file)
        file_match = file_lang_pattern.fullmatch(fullpath)
        if not isfile(fullpath) or file_match is None :
            continue
        producer_type = file_match.group(1)
        metrics: list[tuple] = []
        print(f"Generating metrics for {producer_type}")

        report = extract_logs(fullpath, metrics)
        if report:
            csv_report(metric_output_csv, final_output_csv, producer_type, metrics, report)


def extract_logs(file_name, metrics) -> tuple:
    file = open(file_name, 'r')
    for line in file.readlines():
        if extract_metrics(line, metrics):
            continue
        report = extract_report(line)
        if report:
            return report
    return ()


def extract_metrics(line: str, metrics: list[tuple]):
    res = metrics_pattern.fullmatch(line.strip())
    if res is None:
        return False
    metrics.append(res.groups())
    return True


def extract_report(line: str):
    res = report_pattern.fullmatch(line.strip())
    if res is None:
        return ()
    return res.groups()


def csv_report(metric_output_csv: str, final_output_csv: str, producer_type: str,  metrics: list[tuple], report: tuple):
    with open(metric_output_csv, 'a') as f:
        i = 0
        for metric in metrics:
            ignore = True
            for m in metric:
                if ignore and not (m == 0 or m == 'NaN' or m == 0.0 or m == '0' or m == '0.0'):
                    ignore = False
            if not ignore:
                print(producer_type+',' + str(i) + ',', file=f, end='')
                print(','.join(metric), file=f)
                i += 1 

    with open(final_output_csv, 'a') as f:
        print(producer_type+',', file=f, end='')
        print(','.join(report), file=f)

# ENTRYPOINT
if __name__ == '__main__':
    main(sys.argv[1:])
