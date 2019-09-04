import matplotlib
import matplotlib.pyplot as plt
import numpy as np

def load_from_file(filename):
    result = [];    
    try:
        print('Loading ' + filename + '...');
        file = open(filename, "r")
        while True:
            step = np.fromfile(file, dtype=np.uint64, count=1)
            # check end of file
            if step.size < 1:
                return result;
            step = step[0]
            atom_num = np.fromfile(file, dtype=np.uint64, count=1)[0]
            ids = []; gens = []; vec = []
            for i in range(0, atom_num):
                gens.append(np.fromfile(file, dtype=np.int32, count=1))
                ids.append(np.fromfile(file, dtype=np.uint32, count=1))
                vec.append(np.fromfile(file, dtype=np.float64, count=3))
            result.append({'step': step, 'atom_num': atom_num, 'vec': vec})
    finally:
        file.close()