import re
import sys
from os import listdir
from os.path import isfile, join


config_pattern = re.compile('\\s+([\\w.]+)\\s*=\\s*([\\w.]+)\\s*')
extracted_metrics = [
    'Sent rate',
    'duration spent in queue',
    'batch size',
    'request rate',
    'request latency avg',
    'records per ProduceRequest'
]
metric_patterns = [f"{metric}\\s*=\\s*(NaN|\\d*\\.?\\d*)" for metric in extracted_metrics]
metrics_pattern = re.compile('.*' + '.*'.join(metric_patterns))
report_pattern = re.compile('.*REPORT.*\\s*Produced\\s*(\\d+)\\s+with\\s+(\\d+)\\s+ProduceRequest\\s+in\\s+(.*)')
file_lang_pattern = re.compile('.*/(.*)-producer.*[.]txt')
config_to_print = [
    'batch.size',
    'linger.ms',
    'max.in.flight.requests.per.connection',
]


def main(args):
    results_folder = args[0]
    for file in listdir(results_folder):
        if not isfile(join(results_folder, file)):
            continue
        params: list[(str, str)] = []
        metrics: list[tuple] = []
        report = extract_logs(join(results_folder, file), params, metrics)
        csv_report(params, metrics, report)


def extract_logs(file_name, params, metrics) -> tuple:
    file_match = file_lang_pattern.fullmatch(file_name)
    if file_match is None:
        return ()
    lang = file_match.group(1)
    params.append(('lang', lang))
    file = open(file_name, 'r')
    for line in file.readlines():
        if extract_config(line, params):
            continue
        if extract_metrics(line, metrics):
            continue
        report = extract_report(line)
        if report:
            return report
    return ()


def extract_config(line, params) -> bool:
    res = config_pattern.fullmatch(line)
    if res is None:
        return False
    (key, value) = res.groups()
    if key in config_to_print:
        params.append((key, value))
    return True


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


def csv_report(params: list[(str, str)], metrics: list[tuple], report: tuple):
    if params:
        print('------------')
        print('param,value')
        for (key, value) in params:
            print(f'{key},{value}')
    print('------------')
    print(','.join(extracted_metrics))
    for metric in metrics:
        ignore = True
        for m in metric:
            if ignore and not (m == 0 or m == 'NaN' or m == 0.0 or m == '0' or m == '0.0'):
                ignore = False
        if not ignore:
            print(','.join(metric))
    print('------------')
    print('nb_msgs, nb_requests, duration')
    print(','.join(report))
    print('------------')


# ENTRYPOINT
if __name__ == '__main__':
    main(sys.argv[1:])
