import matplotlib
import matplotlib.pyplot as plt
import numpy as np

def load_from_file(filename):
    result = [];
    num =0
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
            for i in range(0, 1000):
                gens.append(np.fromfile(file, dtype=np.int32, count=1))
                ids.append(np.fromfile(file, dtype=np.uint32, count=1))
                vec.append(np.fromfile(file, dtype=np.float64, count=3))
            result.append({'step': step, 'atom_num': atom_num, 'vec': vec})
            print("frame",num)
            num +=1
    finally:
        file.close()


pos_result = load_from_file("vel.txt")
pos = [result['vec'] for result in pos_result]
vel = np.array(pos)

print(vel)

n, bins, patches = plt.hist(vel[:,1], 50, density=True, facecolor='g', alpha=0.75)


plt.xlabel('Smarts')
plt.ylabel('Probability')
plt.title('Histogram of IQ')
plt.text(60, .025, r'$\mu=100,\ \sigma=15$')
plt.xlim(-10, 10)
plt.ylim(0, 0.03)
plt.grid(True)
plt.show()
