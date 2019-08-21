import yaml
import csv
from yamlvectorconversion import vector_to_yaml

def print_file_yaml(data,timestep):
   
    output = vector_to_yaml(data,timestep)
                    

    with open('C:/Users/Pisuns/rustproject/MOT/example.yaml', 'w') as outfile:
        yaml.dump(output, outfile, default_flow_style=False)


def executing_sim():
    import subprocess
    nothing = subprocess.check_output(["powershell.exe","cd C:/Users/Pisuns/rustproject/MOT ; cargo run --release"],stderr=subprocess.STDOUT)
    with open('C:/Users/Pisuns/rustproject/MOT/output.csv') as csvfile:
        readCSV = csv.reader(csvfile, delimiter=',')
        for row in readCSV:
            result = [row[0],row[1],row[2],row[3]]

    return result
