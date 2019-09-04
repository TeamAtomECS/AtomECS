# -*- coding: utf-8 -*-
"""
Runs an example and plots the result.
"""
import numpy as np
from load import load_from_file
import matplotlib
import matplotlib.pyplot as plt

position_results = load_from_file('pos.dat')
velocity_results = load_from_file('vel.dat')

# Get the positions from the data list.
pos = [result['vec'] for result in position_results]
vel = [result['vec'] for result in velocity_results]

# Collapse list of arrays into one array
pos = np.array(pos)
vel = np.array(vel)

# Create a plot showing the results
fig, ax = plt.subplots()
ax.plot(pos[:,:,2], vel[:,:,2])

ax.set(xlabel='z (m)', ylabel='v_z (m)')
ax.grid()
plt.show()
