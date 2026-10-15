use megastore::graph::Kind;
use megastore::recommendation::recommend;
use megastore::store::Store;

#[test]
fn cadastro_e_consulta() {
    let mut s = Store::new();
    let p = s.add_product("X", "Cat", 9.99);
    assert_eq!(s.product(p).unwrap().name, "X");
    assert_eq!(s.graph.kind(p), Some(Kind::Product));
}

#[test]
fn categoria_nao_duplica() {
    let mut s = Store::new();
    let a = s.add_product("A", "Eletro", 1.0);
    let b = s.add_product("B", "Eletro", 1.0);
    assert_eq!(s.graph.neighbors(a)[0].target, s.graph.neighbors(b)[0].target);
}

#[test]
fn recomenda_similar_nao_comprado() {
    let mut s = Store::new();
    let ntb = s.add_product("Notebook", "E", 5000.0);
    let mon = s.add_product("Monitor",  "E", 2000.0);
    let alice = s.add_customer("Alice");
    s.purchase(alice, ntb, 5.0);
    s.similar(ntb, mon, 2.0);

    let out = recommend(&s.graph, alice, 3, 5);
    let ids: Vec<_> = out.iter().map(|r| r.product).collect();
    assert!(ids.contains(&mon));
    assert!(!ids.contains(&ntb), "não recomendar o que já foi comprado");
}

#[test]
fn sem_duplicatas_e_ordenado() {
    let mut s = Store::new();
    let a = s.add_product("A", "C", 1.0);
    let b = s.add_product("B", "C", 1.0);
    let c = s.add_product("C", "C", 1.0);
    let alice = s.add_customer("Alice");
    s.purchase(alice, a, 5.0);
    s.similar(a, b, 3.0);
    s.similar(b, c, 0.5);

    let out = recommend(&s.graph, alice, 3, 10);
    let mut ids: Vec<_> = out.iter().map(|r| r.product).collect();
    let n = ids.len();
    ids.sort(); ids.dedup();
    assert_eq!(n, ids.len(), "sem duplicatas");
    for w in out.windows(2) { assert!(w[0].score >= w[1].score); }
}