f = open("vel.txt","r")
fp = open("plot.txt","w")
fl = f.readlines()
i_start= 0
i_end= 0
for line in fl:
    data = []
    for i in range(len(line)):
        if line[i] == "(":
            i_start = int(i)
        if line[i] == ")":
            i_end = int(i)
    fp.write(line[i_start+1:i_end]+"\n")
fp.close()
f.close()
        
