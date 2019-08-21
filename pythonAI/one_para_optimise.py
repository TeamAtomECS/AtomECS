from yamloutput import print_file_yaml
from yamloutput import executing_sim

import matplotlib.pyplot as plt
import subprocess
import numpy

lower = -0.05
higher = 0.05
gap = (higher - lower)/50
result = 0 
variable = lower
x = []
y = []
for i in range(50):
    data =[-50417617.14956896, 0.02446864744800686, 146246985.30345988, 0.016109991722513883, 4e-3,4e-3, 20.499099116264183, 0.,0.]
    data [-1] = variable

    print_file_yaml(data,1e-5)
    #print("timestep",rg[i],"result",executing_sim()[0])
    x.append(variable)
    new_y =float(executing_sim()[0])
    
    y.append(new_y)
    if new_y < 500 :
        result = variable
    print(new_y)
    print("step",i)
    variable += gap
print("x",x)
print("y",y)
print("threshold velocity",result)
plt.plot(x,y)
plt.show()

