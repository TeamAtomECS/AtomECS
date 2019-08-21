import GPy
import GPyOpt
import sys
import subprocess
import numpy as np
from numpy.random import seed
import matplotlib
import yaml
from yamloutput import print_file_yaml
from yamloutput import executing_sim
from yamlvectorconversion import yaml_to_vector
from yamlvectorconversion import vector_to_yaml
from record_data import record_csv


domain =[{'name': 'detuning_push', 'type': 'continuous', 'domain': (2e7,2e8)}
         ,{'name': 'e_radius_push', 'type': 'continuous', 'domain': (0,0.03)}
         ,{'name': 'detuning', 'type': 'continuous', 'domain': (2e7,3e8)}
         ,{'name': 'e_radius', 'type': 'continuous', 'domain': (0,0.03)}
         ,{'name': 'radius', 'type': 'continuous', 'domain': (0,0.02)}
         ,{'name': 'thickness', 'type': 'continuous', 'domain': (0,0.02)}
         ,{'name': 'gradient', 'type': 'continuous', 'domain': (10,100)}
         ,{'name': 'height_intersection', 'type': 'continuous', 'domain': (-3e-4,3e-4)}
         ]
X_init = np.array([[1e8,0.003,150e6,0.005,0.001,0.001,63,0.00],
                 [74947405,0.0125,158978861,0.02535,0.0148,0.0039,25.4,-0.000157],
                   [98417617.14956896,0.011188087263972508,150940950.94767362,0.013136734542987792,0.011039319521841085,0.0015885567403108424,26.536042451236924,0.00014243520427615833],
                   [122762162.03324063,0.0071698941070660015,223075681.06744906,0.012659856848946893,0.01650432585781803,0.011216995269314552,40.27404328239832,-0.00024873285945224146],
                    [142423443.93687892,0.009985082868740376,201064043.66313425,0.018411068126509094,0.011774520070475466,0.016159624223898553,32.30086106042445,-0.00014336680217153883],
                    [104106891.01109165,0.02919409059781102,206826642.41109532,0.02322273233728353,0.011286438938507424,0.0009320877468207889,24.73973719150176,-1.1049623083217138e-05],
                    [93940330.75559452,0.025794674244201482,181700672.81536484,0.007771360493106263,0.002958326758065393,0.0017356208201406175,50.719157140234046,0.00029151100789092445]
                   ])
Y_init = np.array([[-12.0],[-13.0],[-22.0],[-25.0],[-25.],[-15.],[-25.]])
#print_file_yaml(X_init[2])
#while True:
    #x=1+1
    


iter_count = 100
current_iter = 0
X_step = X_init
Y_step = Y_init

def criteria(input):
    result = -float(input[0])
    # this is an example only
    # what do we want to minimize, it should be the combination of the two variables: flow and mean velocity
    return result

while current_iter < iter_count:
    bo_step = GPyOpt.methods.BayesianOptimization(f = None, domain = domain, X = X_step, Y = Y_step,acquisition_type='MPI')
    x_next = bo_step.suggest_next_locations()
    
    print(x_next[0])
    
    #this block exectuing the function and get feedback
    
    print_file_yaml(x_next[0])
    y_next = criteria(executing_sim())

    
    desired_outcome = -10.0
    if y_next < desired_outcome:
        record_csv(x_next[0].tolist())
    print("y received:",y_next)
    print("\n")
    y_next = float(y_next)
    X_step = np.vstack((X_step, x_next))
    Y_step = np.vstack((Y_step, y_next))
    
    current_iter += 1

print(x_next)
print("result{}", y_next)


