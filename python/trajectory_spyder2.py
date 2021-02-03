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

name = ""
max_steps = 100000;


data = pd.read_csv('D:\\AION_Git\\AtomECS\\pos'+name+ '.txt', sep=" ", header=None)
N=len(data.set_index(0).loc["step-"+str(max_steps)+",":,1])-1

print("simulation of " + str(N) + " survivors")

list_of_survivors =  np.array(data.iloc[data.shape[0]-N:data.shape[0],0])

print(list_of_survivors)

df = data.set_index(0)

array =  np.array(df.loc[list_of_survivors,1])


#####################

vel_data = pd.read_csv('D:\AION_Git\\AtomECS\\vel'+name+ '.txt', sep=" ", header=None)
vel_df = vel_data.set_index(0)

vel_array =  np.array(vel_df.loc[list_of_survivors,1])

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
vel_data = np.array([to_float_array(i) for i in vel_array]).transpose()


no_of_steps=int(traj_data.shape[1]/N)

def sort_it(unsorted):
    
    x = np.array([[unsorted[0][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    y = np.array([[unsorted[1][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    z = np.array([[unsorted[2][i*no_of_steps + j] for i in range(0,N)] for j in range(0,no_of_steps)] ).flatten()
    
    return np.array([x,y,z])


traj_data = sort_it(traj_data)
vel_data = sort_it(vel_data)



def update_graph(num):

    graph._offsets3d = traj_data[0:3, N*num:N*num+N]
    title.set_text('3D Test, time={}'.format(num))


def get_speeds(velocities, N, num):
    
    return [np.linalg.norm(vel_data[0:3,i]) for i in range(0,N)]


initial_speeds = get_speeds(vel_data, N, 0)


fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')
title = ax.set_title('3D Test')



# Setthe axes properties
ax.set_xlim3d([-0.0005, 0.0005])
ax.set_xlabel('X')

ax.set_ylim3d([-0.0005, 0.0005])
ax.set_ylabel('Y')

ax.set_zlim3d([-0.0005, 0.0005])
ax.set_zlabel('Z')


graph = ax.scatter(traj_data[0:3, 0:0+N][0],traj_data[0:3, 0:0+N][1], traj_data[0:3, 0:0+N][2], s=20, c=initial_speeds, cmap="plasma")
fig.colorbar(graph).ax.set_ylabel("initial speed in m/s")
ani = matplotlib.animation.FuncAnimation(fig, update_graph, 999, 
                               interval=1, blit=False)

Writer = matplotlib.animation.writers['ffmpeg']
writer = Writer(fps=15, metadata=dict(artist='Me'), bitrate=1800)

ani.save('red_mot_steady_state.mp4', writer=writer, dpi=400)

plt.show()