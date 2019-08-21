import csv
import matplotlib.pyplot as plt
def test():
    x=[[],[],[],[],[],[],[],[]]
    v=[[],[],[],[],[],[],[],[]]
    with open('E:\Summer proejct 2019\\result\\phase_diagram\\angle\\book.csv') as csvfile:
        readCSV = csv.reader(csvfile, delimiter=',')
        for row in readCSV:
            for i in range(6):
                if row[2*i]!='':
                    x[i].append(float(row[2*i]))
                    v[i].append(float(row[2*i+1]))
    for i in range(6):
        plt.plot(x[i],v[i])
    plt.show()

test()
