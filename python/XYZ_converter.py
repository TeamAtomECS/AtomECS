# -*- coding: utf-8 -*-
"""
Created on Fri Nov  6 10:26:50 2020

@author: Maurice Zeuner
"""
import numpy as np
from matplotlib import pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import matplotlib.animation
import pandas as pd

name = "_nice"
max_steps = 19980;


data = pd.read_csv('D:\\AION_Git\\AtomECS\\pos'+name+ '.txt', sep=" ", header=None)
N=len(data.set_index(0).loc["step-"+str(max_steps)+",":,1])-1

print("simulation of " + str(N) + " survivors")

list_of_survivors =  np.array(data.iloc[data.shape[0]-N:data.shape[0],0])

#print(list_of_survivors)

df = data.set_index(0)

array =  np.array(df.loc[list_of_survivors,1])


#######################


def to_float_array(stringdata):
    comma_pos = []
    counter = 0
    
    for i in range(0, len(stringdata)):
        if stringdata[i] == ',':
            comma_pos.append(i)
        counter = counter + 1
    return np.array([float(stringdata[1:comma_pos[0]]), float(stringdata[comma_pos[0]+1:comma_pos[1]]), float(stringdata[comma_pos[1]+1:-1])])
    
traj_data = np.array([to_float_array(i) for i in array]).transpose()
print(traj_data)

no_of_steps=int(traj_data.shape[1]/N)

def sort_it(unsorted):
    
    x = np.array([[unsorted[0][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    y = np.array([[unsorted[1][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    z = np.array([[unsorted[2][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    
    return np.array([x,y,z])


traj_data = sort_it(traj_data)

path="D:\\AION_Git\\AtomECS\\output\\"


xyz_file = open(path+"nice_pos"+name+".xyz", "w")
scale = 200
for i in range(0,999):
    xyz_file.write(str(N)+"\n \n")
    for j in range(0,N):
        xyz_file.write("{}\t {}\t {}\t {}\n".format("Sr",scale*traj_data[0][i*N+j],scale*traj_data[1][i*N+j],scale*traj_data[2][i*N+j]))

print("writing done!")


