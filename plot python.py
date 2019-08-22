import csv
import matplotlib.pyplot as plt
def test():
    x=[[],[],[],[],[],[],[],[]]
    v=[[],[],[],[],[],[],[],[]]
    with open('C:\\Users\\Pisuns\\rustproject\\MOT\\asd.csv') as csvfile:
        readCSV = csv.reader(csvfile, delimiter=',')
        for row in readCSV:
            for i in range(6):
                if row[2*i]!='' and row[2*i]!='0':
                    x[i].append(float(row[2*i]))
                    v[i].append(float(row[2*i+1]))
    for i in range(6):
        plt.plot(x[i],v[i])
    plt.show()

test()
