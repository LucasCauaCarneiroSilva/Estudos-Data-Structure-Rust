# ConectaStore — Sistema de Recomendação de Produtos Baseado em Grafos

Projeto da disciplina **Data Structure Strategy and Implementation** — MegaStore.
Implementado em **Rust puro (apenas biblioteca padrão)**.

---

## 1. Objetivo e funcionamento

A MegaStore tem um catálogo grande e recomendações genéricas (mais vendidos ou
mesma categoria), que ignoram as relações reais entre clientes, compras,
categorias e produtos.

O ConectaStore modela esse domínio como um **grafo**:

- **Vértices**: clientes, produtos e categorias.
- **Arestas ponderadas**: compras, visualizações, avaliações, pertencimento a
  categoria e similaridade entre produtos.

Dado um cliente **ou** um produto de partida, um **BFS ponderado com decaimento
por profundidade** percorre as conexões e devolve uma lista ordenada de produtos
relevantes, já sem duplicatas e já excluindo itens que o cliente conhece.

A pergunta do enunciado — *como usar grafos e estruturas complementares para
gerar recomendações relevantes em um catálogo grande* — é respondida assim: o
grafo guarda a **topologia** (quem se liga a quem e com que força) e os
`HashMap` guardam os **atributos e índices**, de modo que cada recomendação
custa O(V+E) **apenas na vizinhança alcançada**, e não uma varredura do catálogo.

---

## 2. Tecnologias e estruturas de dados

| Estrutura | Onde | Uso |
|---|---|---|
| `HashMap<NodeId, Node>` | `graph.rs` | lookup O(1) de vértice por id |
| `HashMap<NodeId, Vec<Edge>>` | `graph.rs` | **lista de adjacência** (grafo esparso) |
| `HashMap<NodeId, Product>` | `store.rs` | atributos ricos fora do grafo |
| `HashMap<String, NodeId>` | `store.rs` | índice de categorias (evita duplicar vértice) |
| `VecDeque<NodeId>` | `recommendation.rs` | fila FIFO do BFS |
| `HashSet<NodeId>` | `recommendation.rs` | exclusão O(1) de itens já conhecidos |
| `Vec<Rec>` + `sort_by` | `recommendation.rs` | ranking final |

- Linguagem: **Rust 2021** (testado com `cargo 1.97.0`).
- **Zero dependências externas** — só `std`.

---

## 3. Arquitetura

```
megastore/
├── src/
│   ├── main.rs            → demonstração + benchmark
│   ├── lib.rs             → declaração dos módulos
│   ├── graph.rs           → topologia (Node, Edge, Graph, BFS primitives)
│   ├── store.rs           → fachada de cadastro + índices de consulta
│   └── recommendation.rs  → BFS ponderado e ranking
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
└── README.md
```

Separação de responsabilidades:

- `graph.rs` não sabe o que é um produto — só conhece `NodeId`, `Kind` e peso.
- `store.rs` é a camada de domínio: cadastra, indexa e traduz operações de
  negócio (`purchase`, `view`, `rate`, `similar`) em arestas.
- `recommendation.rs` depende só de `Graph`, então o algoritmo é testável
  isoladamente, sem montar uma loja inteira.

---

## 4. Compilação e execução

```bash
cargo build --release
cargo run --release
```

## 5. Execução dos testes

```bash
cargo test
```

Resultado esperado: **15 testes passando** (11 unitários + 4 de integração).

```
running 11 tests   (unitários: graph, store, recommendation)   ... ok
running 4 tests    (integração: tests/integration_tests.rs)    ... ok
```

---

## 6. Exemplos de uso

### 6.1 Como biblioteca

```rust
use megastore::store::Store;
use megastore::recommendation::recommend;

let mut s = Store::new();

// cadastro (a categoria é criada/reaproveitada automaticamente)
let ntb = s.add_product("Notebook Gamer", "Eletrônicos", 5999.0);
let mon = s.add_product("Monitor 4K",     "Eletrônicos", 2499.0);
let sup = s.add_product("Suporte Notebook", "Acessórios",  89.0);

// consulta
let p = s.product(ntb).unwrap();
println!("{} — R$ {:.2}", p.name, p.price);
assert_eq!(s.product_by_name("Monitor 4K"), Some(mon));

// conexões
let alice = s.add_customer("Alice");
s.purchase(alice, ntb, 5.0);   // compra (peso = relevância)
s.view(alice, mon);            // visualização (peso fixo 0.3)
s.rate(alice, ntb, 4.5);       // avaliação
s.similar(ntb, sup, 2.5);      // similaridade produto-produto

// recomendação a partir de um CLIENTE (max_depth = 3, top_n = 5)
for r in recommend(&s.graph, alice, 3, 5) {
    println!("{} score={:.2} dist={}", s.product(r.product).unwrap().name, r.score, r.distance);
}

// recomendação a partir de um PRODUTO ("quem viu isso também viu")
for r in recommend(&s.graph, ntb, 2, 5) {
    println!("{} score={:.2}", s.product(r.product).unwrap().name, r.score);
}
```

### 6.2 Saída real de `cargo run --release`

```
=== ConectaStore ===  nós=16  arestas=50

Recomendações para Alice:
  Suporte Notebook   R$   89.00  score= 1.75  dist=2
  Webcam HD          R$  299.00  score= 0.33  dist=3

Similares ao Notebook:
  Monitor 4K         score= 2.65
  Suporte Notebook   score= 2.50
  Mouse Sem Fio      score= 2.00
  Teclado Mecânico   score= 2.00
  Livro Rust         score= 2.00
```

Leitura do resultado: a Alice comprou Notebook, Mouse, Teclado e Livro e
visualizou Monitor e Headset. O sistema **não repete nenhum desses** e sugere o
*Suporte Notebook* (1.75), que ela nunca viu mas está ligado a dois itens dela
(Notebook por similaridade 2.5 e Livro por similaridade 1.0) — exatamente a
venda cruzada que o catálogo plano não encontrava.

> `arestas=50` conta arestas **dirigidas**: cada relação bidirecional ocupa duas
> entradas na lista de adjacência. São 25 relações reais.

---

## 7. Resultados dos testes de desempenho

Medido com `std::time::Instant` em `--release`, uma chamada de
`recommend(cliente, max_depth=3, top_n=10)` sobre um grafo sintético
(`n` produtos, 50 categorias, `n/10` compras):

| Produtos | Arestas dirigidas | Tempo (execução 1) | Tempo (execução 2) | Tempo (execução 3) |
|---:|---:|---:|---:|---:|
| 100 | 220 | 9.98 µs | 8.18 µs | 8.91 µs |
| 1.000 | 2.200 | 217.9 µs | 213.8 µs | 228.3 µs |
| 10.000 | 22.000 | 3.44 ms | 3.46 ms | 2.95 ms |
| 50.000 | 110.000 | 21.55 ms | 22.17 ms | 21.27 ms |

De 100 → 50.000 produtos (500×) o tempo cresce ~2.400×. O crescimento é
aproximadamente linear em `V+E`, com a folga explicada pelo vértice de
categoria, que funciona como *hub*: com 50 categorias fixas, cada categoria
acumula `n/50` produtos e o BFS a partir dela expande uma vizinhança que cresce
junto com `n`. Reproduza com `cargo run --release`.

---

## 8. Vídeo pitch

🎥 **Link:** _(a preencher antes da entrega — YouTube, pode ser "não listado")_

---

## 9. Complexidade

| Operação | Tempo | Espaço |
|---|---|---|
| Cadastro de produto/cliente/categoria | O(1) amortizado | O(1) |
| Consulta por id | O(1) | O(1) |
| Consulta por nome | O(P) | O(1) |
| Vizinhança de um vértice | O(grau) | O(1) |
| Recomendação (BFS ponderado) | O(V+E) + O(k log k) do ranking | O(V) |
| Grafo completo em memória | — | O(V+E) |

`k` = número de produtos candidatos encontrados pelo percurso.
