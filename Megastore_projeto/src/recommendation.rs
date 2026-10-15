use std::collections::{HashMap, HashSet, VecDeque};
use crate::graph::{Graph, Kind, NodeId};

#[derive(Debug, Clone)]
pub struct Rec { pub product: NodeId, pub score: f64, pub distance: usize }

/// BFS ponderado. Score de cada vizinho = peso / (profundidade + 1).
/// - Se `start` é cliente: exclui produtos já ligados a ele (comprados/vistos/avaliados).
/// - Se `start` é produto: exclui ele mesmo.
pub fn recommend(graph: &Graph, start: NodeId, max_depth: usize, top_n: usize) -> Vec<Rec> {
    let exclude: HashSet<NodeId> = if graph.kind(start) == Some(Kind::Customer) {
        graph.neighbors(start).iter()
            .filter(|e| graph.kind(e.target) == Some(Kind::Product))
            .map(|e| e.target)
            .collect()
    } else {
        [start].into_iter().collect()
    };

    let mut visited: HashMap<NodeId, usize> = HashMap::new();
    let mut scores: HashMap<NodeId, f64> = HashMap::new();
    let mut queue = VecDeque::new();

    visited.insert(start, 0);
    queue.push_back(start);

    while let Some(cur) = queue.pop_front() {
        let depth = visited[&cur];
        if depth >= max_depth { continue; }
        for e in graph.neighbors(cur) {
            *scores.entry(e.target).or_insert(0.0) += e.weight / ((depth + 1) as f64);
            if !visited.contains_key(&e.target) {
                visited.insert(e.target, depth + 1);
                queue.push_back(e.target);
            }
        }
    }

    let mut out: Vec<Rec> = scores.into_iter()
        .filter(|(id, _)| !exclude.contains(id) && graph.kind(*id) == Some(Kind::Product))
        .map(|(id, score)| Rec { product: id, score, distance: visited[&id] })
        .collect();

    out.sort_by(|a, b| {
        b.score.partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.product.cmp(&b.product))     // desempate estável por id
    });
    out.truncate(top_n);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    fn grafo_linear() -> (Graph, NodeId, NodeId, NodeId) {
        let mut g = Graph::new();
        g.add_node(1, Kind::Customer, "c");
        g.add_node(2, Kind::Product, "p2");
        g.add_node(3, Kind::Product, "p3");
        g.add_edge(1, 2, 4.0, true); // cliente comprou p2
        g.add_edge(2, 3, 2.0, true); // p2 ~ p3
        (g, 1, 2, 3)
    }

    #[test]
    fn exclui_produto_ja_ligado_ao_cliente() {
        let (g, c, p2, p3) = grafo_linear();
        let ids: Vec<_> = recommend(&g, c, 3, 10).iter().map(|r| r.product).collect();
        assert!(!ids.contains(&p2));
        assert!(ids.contains(&p3));
    }

    #[test]
    fn partindo_de_produto_exclui_ele_mesmo() {
        let (g, _, p2, p3) = grafo_linear();
        let ids: Vec<_> = recommend(&g, p2, 2, 10).iter().map(|r| r.product).collect();
        assert!(!ids.contains(&p2));
        assert!(ids.contains(&p3));
    }

    #[test]
    fn max_depth_limita_o_alcance() {
        let (g, c, _, p3) = grafo_linear();
        // p3 está a 2 saltos do cliente; com max_depth=1 não deve aparecer
        let ids: Vec<_> = recommend(&g, c, 1, 10).iter().map(|r| r.product).collect();
        assert!(!ids.contains(&p3));
    }

    #[test]
    fn top_n_trunca_o_resultado() {
        let mut g = Graph::new();
        g.add_node(1, Kind::Customer, "c");
        for i in 2..=8 { g.add_node(i, Kind::Product, "p"); g.add_edge(1, i, i as f64, true); }
        g.add_node(99, Kind::Product, "alvo");
        g.add_edge(2, 99, 1.0, true);
        assert!(recommend(&g, 1, 3, 1).len() <= 1);
    }

    #[test]
    fn empate_de_score_e_desempatado_por_id() {
        let mut g = Graph::new();
        g.add_node(1, Kind::Customer, "c");
        g.add_node(2, Kind::Product, "comprado");
        g.add_edge(1, 2, 1.0, true);
        // dois produtos com peso idêntico -> empate perfeito
        g.add_node(10, Kind::Product, "x");
        g.add_node(11, Kind::Product, "y");
        g.add_edge(2, 10, 3.0, true);
        g.add_edge(2, 11, 3.0, true);

        for _ in 0..50 {
            let ids: Vec<_> = recommend(&g, 1, 3, 10).iter().map(|r| r.product).collect();
            assert_eq!(ids, vec![10, 11], "ordem deve ser estável entre execuções");
        }
    }

    #[test]
    fn score_decai_com_a_profundidade() {
        let (g, c, _, p3) = grafo_linear();
        let r = recommend(&g, c, 3, 10);
        let p3r = r.iter().find(|r| r.product == p3).unwrap();
        assert_eq!(p3r.distance, 2);
        assert!((p3r.score - 1.0).abs() < 1e-9, "2.0 / (1+1) = 1.0, obtido {}", p3r.score);
    }
}
