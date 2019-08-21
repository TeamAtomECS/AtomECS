import random
from deap import creator, base, tools, algorithms
from yamloutput import print_file_yaml
from yamloutput import executing_sim
import numpy as np
# basic registration
creator.create("FitnessMax", base.Fitness, weights=(1.0,))
creator.create("Individual", list, fitness=creator.FitnessMax)

toolbox = base.Toolbox()



def gene_2DMOT(cls):
    import csv
    with open('record.csv') as csvfile:
        reader = csv.reader(csvfile, delimiter=',', quotechar='"')

        result =[]
        luck_threshold = 0.0
        for row in reader:
            luck = random.random()
            if (luck > luck_threshold) & (len(row) > 1):
                luck_threshold = luck
                result = []
                for i in row:
                    result.append(float(i))
    #print(result)
    return cls(result)

toolbox.register("individual",gene_2DMOT, creator.Individual)
toolbox.register("population", tools.initRepeat, list, toolbox.individual)

def evalOneMax(individual):
    print_file_yaml(individual,3e-5)
    ( rate, vx, vy, vz ) = executing_sim()
    result = float(rate)
    print("evaluate")
    #some function that weigh the output to get the fitness TODO
    return result,

toolbox.register("evaluate", evalOneMax)


toolbox.register("mate", tools.cxTwoPoint)
# that is truly the trouble here, I am not sure what kind of mutation is best for this simulation
# I will probably use a gaussian or shrink
# need to test whether those method work for the array gene as well
# anyway I think the best idea is to get a new mutate function on my own


toolbox.register("select", tools.selTournament, tournsize=3)

def mutate(data,prob):
    result = []
    for i in data:
        luck = random.random()
        if luck < prob:
            change = random.random() -0.4
            i_mutate = i * (1 + change/2)
            result.append(i_mutate)
        else:
            result.append(i)
    return creator.Individual(result)
    

def var(population, toolbox, lambda_, cxpb, mutpb):
    assert (cxpb + mutpb) <= 1.0, (
        "The sum of the crossover and mutation probabilities must be smaller "
        "or equal to 1.0.")

    offspring = []
    for _ in range(lambda_):
        op_choice = random.random()
        if op_choice < cxpb:            # Apply crossover
            ind1, ind2 = list(map(toolbox.clone, random.sample(population, 2)))
            ind1, ind2 = toolbox.mate(ind1, ind2)
            del ind1.fitness.values
            offspring.append(ind1)
        elif op_choice < cxpb + mutpb:  # Apply mutation
            ind = toolbox.clone(random.choice(population))
            ind = mutate(ind,0.2)
            ind = creator.Individual(ind)
            del ind.fitness.values
            offspring.append(ind)
        else:                           # Apply reproduction
            offspring.append(random.choice(population))

    return offspring

# this is the default selecting algorithm , will look at it to see if it is working
def test():
    NGEN=40
    population = toolbox.population(n=12)
    fits = toolbox.map(toolbox.evaluate, population)
    hall_of_fame = []
    fame = [0.0]
    for gen in range(NGEN):
        print("gen",NGEN,"\n\n")
        for ind in population:
            print("individual",ind,"fitness",ind.fitness.values)

        offspring = var(population, toolbox, 2*len(population),0.35,0.55)
            
        fits = toolbox.map(toolbox.evaluate, offspring)
        for fit, ind in zip(fits, offspring):
            ind.fitness.values = fit
            if fit[0] > min(fame):
                hall_of_fame.append(ind)
                fame.append(fit[0])
                if len(hall_of_fame) > 10:
                    index = fame.index(min(fame))
                    hall_of_fame.remove(hall_of_fame[index])
                    fame.remove(fame[index])
        print("fame",fame)
        luck = random.random()
        if luck > 0.95:
            population.append(random.choice(hall_of_fame))
            
        population = toolbox.select(offspring, k=len(population))
        best = tools.selBest(population, 1)[0]
        print("best individual {}",best," best performance",best.fitness.values)
    top3 = tools.selBest(population, k=3)

#pop, logbook = algorithms.eaSimple(population, toolbox, cxpb=0.5, mutpb=0.2, ngen=2)
test()
