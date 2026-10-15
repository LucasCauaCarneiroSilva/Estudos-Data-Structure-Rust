use std::collections::HashMap;
use crate::graph::{Graph, Kind, NodeId};

#[derive(Debug, Clone)]
pub struct Product { pub name: String, pub category: String, pub price: f64 }

#[derive(Debug, Default)]
pub struct Store {
    pub graph: Graph,
    pub products: HashMap<NodeId, Product>,
    pub customers: HashMap<NodeId, String>,
    categories: HashMap<String, NodeId>,
    next_id: NodeId,
}

impl Store {
    pub fn new() -> Self { Self { next_id: 1, ..Default::default() } }

    fn next(&mut self) -> NodeId { let id = self.next_id; self.next_id += 1; id }

    pub fn add_customer(&mut self, name: &str) -> NodeId {
        let id = self.next();
        self.graph.add_node(id, Kind::Customer, name);
        self.customers.insert(id, name.into());
        id
    }

    pub fn add_category(&mut self, name: &str) -> NodeId {
        if let Some(&id) = self.categories.get(name) { return id; }
        let id = self.next();
        self.graph.add_node(id, Kind::Category, name);
        self.categories.insert(name.into(), id);
        id
    }

    pub fn add_product(&mut self, name: &str, category: &str, price: f64) -> NodeId {
        let cat = self.add_category(category);
        let id = self.next();
        self.graph.add_node(id, Kind::Product, name);
        self.graph.add_edge(id, cat, 1.0, true);
        self.products.insert(id, Product { name: name.into(), category: category.into(), price });
        id
    }

    pub fn purchase(&mut self, c: NodeId, p: NodeId, w: f64) { self.graph.add_edge(c, p, w, true); }
    pub fn view    (&mut self, c: NodeId, p: NodeId)         { self.graph.add_edge(c, p, 0.3, true); }
    pub fn rate    (&mut self, c: NodeId, p: NodeId, w: f64) { self.graph.add_edge(c, p, w, true); }
    pub fn similar (&mut self, a: NodeId, b: NodeId, w: f64) { self.graph.add_edge(a, b, w, true); }

    pub fn product(&self, id: NodeId) -> Option<&Product> { self.products.get(&id) }

    /// Consulta por nome exato. Em caso de homônimos devolve o menor id,
    /// para que o resultado não dependa da ordem de iteração do `HashMap`.
    pub fn product_by_name(&self, name: &str) -> Option<NodeId> {
        self.products.iter()
            .filter(|(_, p)| p.name == name)
            .map(|(id, _)| *id)
            .min()
    }

    pub fn customer_by_name(&self, name: &str) -> Option<NodeId> {
        self.customers.iter()
            .filter(|(_, n)| n.as_str() == name)
            .map(|(id, _)| *id)
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categoria_reaproveitada_nao_cria_no_novo() {
        let mut s = Store::new();
        s.add_product("A", "Eletro", 1.0);
        s.add_product("B", "Eletro", 1.0);
        // 2 produtos + 1 categoria
        assert_eq!(s.graph.node_count(), 3);
    }

    #[test]
    fn consulta_por_nome_e_deterministica() {
        let mut s = Store::new();
        let a = s.add_product("Notebook Gamer", "E", 1.0);
        s.add_product("Suporte Notebook", "A", 1.0);
        for _ in 0..50 {
            assert_eq!(s.product_by_name("Notebook Gamer"), Some(a));
        }
        assert_eq!(s.product_by_name("Inexistente"), None);
    }

    #[test]
    fn avaliacao_cria_aresta_bidirecional() {
        let mut s = Store::new();
        let p = s.add_product("P", "C", 1.0);
        let c = s.add_customer("C1");
        s.rate(c, p, 4.5);
        assert!(s.graph.neighbors(c).iter().any(|e| e.target == p && e.weight == 4.5));
        assert!(s.graph.neighbors(p).iter().any(|e| e.target == c && e.weight == 4.5));
    }
}