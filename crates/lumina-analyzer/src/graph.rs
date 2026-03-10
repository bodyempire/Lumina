use std::collections::{HashMap, VecDeque};

pub type NodeId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldNode {
    pub entity: String,
    pub field:  String,
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub node_ids:     HashMap<FieldNode, NodeId>,
    pub nodes:        Vec<FieldNode>,
    pub dependents:   Vec<Vec<NodeId>>,
    pub dependencies: Vec<Vec<NodeId>>,
    pub topo_order:   Vec<NodeId>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            node_ids: HashMap::new(),
            nodes: Vec::new(),
            dependents: Vec::new(),
            dependencies: Vec::new(),
            topo_order: Vec::new(),
        }
    }

    pub fn intern(&mut self, entity: &str, field: &str) -> NodeId {
        let node = FieldNode {
            entity: entity.to_string(),
            field: field.to_string(),
        };
        if let Some(&id) = self.node_ids.get(&node) {
            id
        } else {
            let id = self.nodes.len() as u32;
            self.node_ids.insert(node.clone(), id);
            self.nodes.push(node);
            self.dependents.push(Vec::new());
            self.dependencies.push(Vec::new());
            id
        }
    }

    pub fn add_edge(&mut self, dep: NodeId, dependent: NodeId) {
        if !self.dependents[dep as usize].contains(&dependent) {
            self.dependents[dep as usize].push(dependent);
            self.dependencies[dependent as usize].push(dep);
        }
    }

    pub fn get_node(&self, entity: &str, field: &str) -> Option<NodeId> {
        self.node_ids.get(&FieldNode {
            entity: entity.to_string(),
            field: field.to_string(),
        }).copied()
    }

    pub fn compute_topo_order(&mut self) -> Result<(), CycleError> {
        let n = self.nodes.len();
        let mut in_degree = vec![0; n];
        for deps in &self.dependents {
            for &dest in deps {
                in_degree[dest as usize] += 1;
            }
        }

        let mut queue = VecDeque::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push_back(i as u32);
            }
        }

        let mut order = Vec::new();
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &v in &self.dependents[u as usize] {
                in_degree[v as usize] -= 1;
                if in_degree[v as usize] == 0 {
                    queue.push_back(v);
                }
            }
        }

        if order.len() == n {
            self.topo_order = order;
            Ok(())
        } else {
            // Find cycle
            let mut chain = Vec::new();
            // Simple cycle detection for the error message
            // We just report the nodes that weren't visited
            for i in 0..n {
                if in_degree[i] > 0 {
                    chain.push(format!("{}.{}", self.nodes[i].entity, self.nodes[i].field));
                }
            }
            Err(CycleError { chain })
        }
    }

    pub fn recomputation_order(&self, changed: NodeId) -> Vec<NodeId> {
        let mut visited = vec![false; self.nodes.len()];
        let mut result = Vec::new();
        let mut stack = vec![changed];

        while let Some(u) = stack.pop() {
            if !visited[u as usize] {
                visited[u as usize] = true;
                result.push(u);
                for &v in &self.dependents[u as usize] {
                    stack.push(v);
                }
            }
        }

        // Sort by precomputed topo order
        let topo_map: HashMap<NodeId, usize> = self.topo_order.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        result.sort_by_key(|id| topo_map.get(id).unwrap_or(&usize::MAX));
        result
    }
}

#[derive(Debug)]
pub struct CycleError {
    pub chain: Vec<String>,
}
