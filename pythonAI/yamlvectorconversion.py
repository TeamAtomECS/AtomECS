def yaml_to_vector(yaml):
    result = []
    for laser in yaml.lasers:
        result.append(yaml.frequency)
        result.append(yaml.polarization)
        result.append(yaml.e_radius)
        result.append(yaml.power)
        for i in range(3):
            result.append(yaml.intersection[i])
        for i in range(3):
            result.append(yaml.direction[i])
        # etc, more will be written when the rust side is settled
        
    return result

def vector_to_yaml(vector,timestep):
    frequency_push = 3e8 / 461.0e-9 - float(vector[0])
    e_radius_push = float(vector[1])
    frequency_cool = 3e8 / 461.0e-9 - float(vector[2])
    e_radius_cool = float(vector[3])
    intersection_cool = float(vector[7])
    yaml_result ={
                "lasers":[{
                        "direction":[1.0,0.0,0.0],
                        "frequency":frequency_push,
                        "polarization":1.0,
                        "power":0.045,
                        "e_radius":e_radius_push,
                        "intersection":[0.0,0.0,0.0],
                    },
                    {
                        "direction":[0.0,1.0,1.0],
                        "frequency":frequency_cool,
                        "polarization":-1.0,
                        "power":0.235,
                        "e_radius":e_radius_cool,
                        "intersection":[0.0,0.0,intersection_cool],
                        },
                    {
                        "direction":[0.0,-1.0,-1.0],
                        "frequency":frequency_cool,
                        "polarization":-1.0,
                        "power":0.235,
                        "e_radius":e_radius_cool,
                        "intersection":[0.0,0.0,intersection_cool],
                        },
                    {
                        "direction":[0.0,-1.0,1.0],
                        "frequency":frequency_cool,
                        "polarization":-1.0,
                        "power":0.235,
                        "e_radius":e_radius_cool,
                        "intersection":[0.0,0.0,intersection_cool],
                        },
                    {
                        "direction":[0.0,1.0,-1.0],
                        "frequency":frequency_cool,
                        "polarization":-1.0,
                        "power":0.235,
                        "e_radius":e_radius_cool,
                        "intersection":[0.0,0.0,intersection_cool],
                        },
                    ],
                "ovens":[{"position":[0.,0.,-0.15],
                            "direction":[0.,0.,1.0],
                            "temperature":vector[8],
                            "rate":0.,
                            "instant_emission":2000,
                            "radius_aperture":float(vector[4]),
                            "thickness":float(vector[5])}
                       ],
                "atominfo":{ "mup": 9.274e-24,
                              "mum": -9.274e-24,
                              "muz": 0.0,
                              "frequency": 3e8 / 461.0e-9,
                              "linewidth": 32e6,
                             "saturation_intensity": 430},
                "magnetic":{
                                  "centre":[0.,0.,0.],
                                
                                  "gradient": float(vector[6]),
                                  "uniform":[0.,0.,0.00002]},
                "mass":{"distribution":[{"mass":86,"ratio":0.0986},
                                          {"mass":87,"ratio":0.07},
                                        {"mass":88,"ratio":0.8258}],
                        "normalised":False
                        },
                "detector":{"radius": 0.02,
                            "thickness" : 0.005,
                            "position": [0.04,0.0,0.0],
                            "direction":[1.0,0.,0.],
                     },
                "timestep":timestep

                            
                    }
    return yaml_result

