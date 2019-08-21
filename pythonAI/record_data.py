import csv
def record_csv(data):
    with open('record.csv', mode='a') as record_file:
        writer = csv.writer(record_file, delimiter=',', quotechar='"', quoting=csv.QUOTE_MINIMAL)

        writer.writerow(data)
