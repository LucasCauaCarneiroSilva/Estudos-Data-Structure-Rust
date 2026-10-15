use std::collections::HashMap;

pub type NodeId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { Customer, Product, Category }

#[derive(Debug, Clone)]
pub struct Node { pub kind: Kind, pub label: String }

#[derive(Debug, Clone)]
pub struct Edge { pub target: NodeId, pub weight: f64 }

#[derive(Debug, Default)]
pub struct Graph {
    pub nodes: HashMap<NodeId, Node>,
    adj: HashMap<NodeId, Vec<Edge>>,
}

impl Graph {
    pub fn new() -> Self { Self::default() }

    pub fn add_node(&mut self, id: NodeId, kind: Kind, label: impl Into<String>) {
        self.nodes.insert(id, Node { kind, label: label.into() });
        self.adj.entry(id).or_default();
    }

    pub fn add_edge(&mut self, a: NodeId, b: NodeId, weight: f64, bidirectional: bool) {
        self.adj.entry(a).or_default().push(Edge { target: b, weight });
        if bidirectional {
            self.adj.entry(b).or_default().push(Edge { target: a, weight });
        }
    }

    pub fn neighbors(&self, id: NodeId) -> &[Edge] {
        match self.adj.get(&id) {
            Some(v) => v.as_slice(),
            None => &[],
        }
    }

    pub fn kind(&self, id: NodeId) -> Option<Kind> {
        self.nodes.get(&id).map(|n| n.kind)
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }

    /// Número de arestas DIRIGIDAS armazenadas em `adj`.
    /// Uma aresta bidirecional conta 2 (ida e volta).
    pub fn edge_count(&self) -> usize { self.adj.values().map(Vec::len).sum() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vizinhos_de_no_inexistente_e_vazio() {
        let g = Graph::new();
        assert!(g.neighbors(999).is_empty());
        assert_eq!(g.kind(999), None);
    }

    #[test]
    fn aresta_unidirecional_vs_bidirecional() {
        let mut g = Graph::new();
        g.add_node(1, Kind::Product, "a");
        g.add_node(2, Kind::Product, "b");
        g.add_node(3, Kind::Product, "c");

        g.add_edge(1, 2, 1.0, false);
        assert_eq!(g.neighbors(1).len(), 1);
        assert_eq!(g.neighbors(2).len(), 0);

        g.add_edge(1, 3, 2.0, true);
        assert_eq!(g.neighbors(3).len(), 1);
        assert_eq!(g.edge_count(), 3); // 1 dirigida + 2 do par bidirecional
    }
}