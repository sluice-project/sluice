import networkx as nx 
import matplotlib.pyplot as plt 
from graphviz import Digraph

f1 = open("dependency_dag.txt",'r')
atoms = f1.read().split('\n')[:-1]

f2 = open("next_prev_nodes.txt", 'r')
next_prev = f2.read().split('\n')[:-1]

next_nodes = [eval(i.split(':')[0]) for i in next_prev]
prev_nodes = [eval(i.split(':')[1]) for i in next_prev]

# graphviz
dag = Digraph(comment='Dependency Dag')

for i in range(len(atoms)):
	if next_nodes[i] == [] and prev_nodes[i] == []:
		continue
	w = atoms[i].replace(':','')
	dag.node(w, atoms[i])


for i in range(len(next_nodes)):
	if next_nodes[i] == [] and prev_nodes[i] == []:
		continue
	for j in next_nodes[i]:
		a = atoms[i].replace(':','')
		b = atoms[j].replace(':','')
		dag.edge(a, b)

print(dag.source)  
dag.render('dep_dag.gv', view=True)  



# # networkx
# G = nx.DiGraph() 
# edges = []

# for i in range(len(next_nodes)):
# 	for j in next_nodes[i]:
# 		edges.append((atoms[i], atoms[j]))


# G.add_edges_from(edges) 
  
# plt.figure(figsize = (9, 9)) 
# nx.draw_networkx(G, with_label = True, node_color ='green', font_size = 9) 
# plt.show()


