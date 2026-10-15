use megastore::recommendation::recommend;
use megastore::store::Store;
use std::time::Instant;

fn demo() -> Store {
    let mut s = Store::new();

    // Eletrônicos
    let ntb = s.add_product("Notebook Gamer",   "Eletrônicos", 5999.0);
    let mon = s.add_product("Monitor 4K",       "Eletrônicos", 2499.0);
    let mse = s.add_product("Mouse Sem Fio",    "Eletrônicos",  149.0);
    let tec = s.add_product("Teclado Mecânico", "Eletrônicos",  399.0);
    let hed = s.add_product("Headset BT",       "Eletrônicos",  499.0);
    s.add_product("Webcam HD",                  "Eletrônicos",  299.0);

    // Vestuário
    let cam = s.add_product("Camiseta",         "Vestuário",     59.0);
    let ten = s.add_product("Tênis Corrida",    "Vestuário",    349.0);

    // Livros e Acessórios
    let lvr = s.add_product("Livro Rust",       "Livros",        89.0);
    let sup = s.add_product("Suporte Notebook", "Acessórios",    89.0);

    let alice = s.add_customer("Alice");
    let bruno = s.add_customer("Bruno");

    // Compras / views
    s.purchase(alice, ntb, 5.0);
    s.purchase(alice, mse, 3.0);
    s.purchase(alice, tec, 3.0);
    s.purchase(alice, lvr, 2.0);
    s.view(alice, mon);
    s.view(alice, hed);

    s.purchase(bruno, cam, 2.0);
    s.purchase(bruno, ten, 3.0);

    // Avaliações (o enunciado cita avaliações como aresta do grafo)
    s.rate(alice, lvr, 1.0);

    // Similaridades produto-produto
    s.similar(ntb, mon, 2.0);
    s.similar(mse, tec, 1.5);
    s.similar(tec, hed, 1.0);
    s.similar(cam, ten, 1.2);
    s.similar(ntb, sup, 2.5);
    s.similar(lvr, sup, 1.0);

    s
}

fn main() {
    let s = demo();
    println!("=== ConectaStore ===  nós={}  arestas={}",
             s.graph.node_count(), s.graph.edge_count());

    let alice = s.customer_by_name("Alice").expect("cliente da demo");
    println!("\nRecomendações para Alice:");
    for r in recommend(&s.graph, alice, 3, 5) {
        let p = s.product(r.product).unwrap();
        println!("  {:<18} R$ {:>7.2}  score={:>5.2}  dist={}",
                 p.name, p.price, r.score, r.distance);
    }

    // Busca por nome EXATO: `contains("Notebook")` casava também com
    // "Suporte Notebook" e a escolha dependia da ordem do HashMap.
    let ntb = s.product_by_name("Notebook Gamer").expect("produto da demo");
    println!("\nSimilares ao Notebook:");
    for r in recommend(&s.graph, ntb, 2, 5) {
        let p = s.product(r.product).unwrap();
        println!("  {:<18} score={:>5.2}", p.name, r.score);
    }

    // ---------- Benchmark ----------
    println!("\n=== Benchmark ===");
    println!("{:>8} | {:>10} | {:>12}", "Produtos", "Arestas", "Tempo");
    for n in [100usize, 1_000, 10_000, 50_000] {
        let mut big = Store::new();
        let mut ids = Vec::with_capacity(n);
        for i in 0..n {
            ids.push(big.add_product(&format!("P{i}"),
                                     &format!("C{}", i % 50),
                                     10.0 + (i as f64) % 100.0));
        }
        let cust = big.add_customer("C");
        for i in 0..(n / 10).max(1) {
            big.purchase(cust, ids[i % n], 1.0 + (i % 5) as f64);
        }
        let t = Instant::now();
        let _ = recommend(&big.graph, cust, 3, 10);
        println!("{:>8} | {:>10} | {:>12?}", n, big.graph.edge_count(), t.elapsed());
    }
}