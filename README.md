# Oxigraph

An untested attempt to port [Oxigraph](https://github.com/oxigraph/oxigraph) to the [Internet Computer](https://internetcomputer.org/), in order to host an RDF database fully on-chain.
The repository is a fork of [oxigraph](https://github.com/oxigraph/oxigraph) and has been adapted to be compiled to [`wasm32-unknown-unknown`](https://doc.rust-lang.org/stable/nightly-rustc/rustc_target/spec/wasm32_unknown_unknown/index.html) Rust target.
For usage reference, see Oxigraph's docs at https://docs.rs/oxigraph.

For this porting, relevant features from Oxigraph are:
- [The database written as a Rust library](https://crates.io/crates/oxigraph). Its source code is in the `lib` directory.
- it implements the following specifications:
  - [SPARQL 1.1 Query](https://www.w3.org/TR/sparql11-query/), [SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/), and [SPARQL 1.1 Federated Query](https://www.w3.org/TR/sparql11-federated-query/).
  - [Turtle](https://www.w3.org/TR/turtle/), [TriG](https://www.w3.org/TR/trig/), [N-Triples](https://www.w3.org/TR/n-triples/), [N-Quads](https://www.w3.org/TR/n-quads/), and [RDF XML](https://www.w3.org/TR/rdf-syntax-grammar/) RDF serialization formats for both data ingestion and retrieval using the [Rio library](https://github.com/oxigraph/rio).
  - [SPARQL Query Results XML Format](http://www.w3.org/TR/rdf-sparql-XMLres/), [SPARQL 1.1 Query Results JSON Format](https://www.w3.org/TR/sparql11-results-json/) and [SPARQL 1.1 Query Results CSV and TSV Formats](https://www.w3.org/TR/sparql11-results-csv-tsv/).

## What's been ported

The main problems for Oxigraph to run on the Internet Computer were the [Date::now()](https://docs.rs/js-sys/latest/js_sys/struct.Date.html#method.now) function from [js_sys](https://crates.io/crates/js_sys) crate and the random number generator from [rand](https://crates.io/crates/rand) crate.

### `Date::now()`
Using [time](https://docs.rs/ic-cdk/latest/ic_cdk/api/fn.time.html) from ic-cdk api, this function has been substituted with:
```rust
pub fn now() -> f64 {
  (ic_cdk::api::time() / 1_000_000) as f64
}
```

### Random Number Generator
Since the randomness required doesn't need to be cryptographically secure (basically used only to generate temporary helpers and blank nodes), substituting it with the current time can be enough:
```rust
use getrandom::{register_custom_getrandom, Error};

fn getrandom_from_timestamp(buf: &mut [u8]) -> Result<(), Error> {
    let timestamp_bytes = ic_cdk::api::time().to_be_bytes();
    buf[..8].copy_from_slice(&timestamp_bytes);
    Ok(())
}

register_custom_getrandom!(getrandom_from_timestamp);
```
> The problem still remains for the UUID generator, becuase the timestamp "random" generation doesn't make it compliant. As a temporary fix, it could be **removed** as a temporary fix, as suggested in [this comment](https://github.com/oxigraph/oxigraph/issues/471#issuecomment-1517703078).

## Help

See [Oxigraph's GitHub discussions](https://github.com/oxigraph/oxigraph/discussions) or [the Oxigraph's Gitter chat](https://gitter.im/oxigraph/community) to ask questions or talk about Oxigraph.
If you want to report bugs, use [Oxigraph's Bug reports](https://github.com/oxigraph/oxigraph/issues).

## License

This project is licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).

### Contribution

This is an untested implementation that needs a lot of help from the community! Feel free to open PRs :).

