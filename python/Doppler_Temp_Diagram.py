# -*- coding: utf-8 -*-
"""
Created on Fri Nov  6 10:26:50 2020

@author: Maurice Zeuner
"""
import numpy as np
from matplotlib import pyplot as plt
import pandas as pd
from scipy import constants

def sort_it(unsorted):
    
    x = np.array([[unsorted[0][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    y = np.array([[unsorted[1][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    z = np.array([[unsorted[2][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    
    return np.array([x,y,z])

def to_float_array(stringdata):
    comma_pos = []
    counter = 0
    
    for i in range(0, len(stringdata)):
        if stringdata[i] == ',':
            comma_pos.append(i)
        counter = counter + 1
    return np.array([float(stringdata[1:comma_pos[0]]), float(stringdata[comma_pos[0]+1:comma_pos[1]]), float(stringdata[comma_pos[1]+1:-1])])

all_temps = []

max_steps = 100000
detunings = ["50", "200", "500", "1000", "2000"]

for p in detunings:
    name = "_"+ p
    
    path = 'D:\\AION_Git\\AtomECS\\output\\detuning_T\\red_Sr_MOT\\'

    data = pd.read_csv(path + 'pos'+name+ '.txt', sep=" ", header=None)
    N=len(data.set_index(0).loc["step-"+str(max_steps)+",":,1])-1

    print("simulation of " + str(N) + " survivors")

    list_of_survivors =  np.array(data.iloc[data.shape[0]-N:data.shape[0],0])

    print(list_of_survivors)

    df = data.set_index(0)

    array =  np.array(df.loc[list_of_survivors,1])


    #####################

    vel_data = pd.read_csv(path + 'vel'+name+ '.txt', sep=" ", header=None)
    vel_df = vel_data.set_index(0)

    vel_array =  np.array(vel_df.loc[list_of_survivors,1])

    #######################



    traj_data = np.array([to_float_array(i) for i in array]).transpose()
    vel_data = np.array([to_float_array(i) for i in vel_array]).transpose()


    no_of_steps=int(traj_data.shape[1]/N)




    traj_data = sort_it(traj_data)
    vel_data = sort_it(vel_data)

    print(vel_data)

    Temp = []

    for i in range(0,  int(max_steps/100)):
        total_vel = 0
        for j in range(0, N):
            total_vel = total_vel + (vel_data[0][i+j]**2+vel_data[1][i+j]**2+vel_data[2][i+j]**2)**0.5
        total_vel = total_vel / N
        Temp.append(88*constants.physical_constants["atomic mass constant"][0] * total_vel**2 / (3*constants.k))

    all_temps.append(Temp)
    


t = np.linspace(0, 1e-1, int(max_steps/100))
for i, temp in enumerate(all_temps):
    plt.plot(t, temp, label = "-"+detunings[i]+" KHz")
#plt.yscale('log')
plt.xlabel("time in s")
plt.ylabel("Temperature in K")
plt.legend()
plt.show()