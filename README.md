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
To register the Random Number Generator you have two options:
- (**default**) register a custom getrandom implementation by yourself and pass a `&RefCell<StdRng>` to the `init()` function, which saves it to the local state. Example:
    ```rust
    use ic_cdk_macros::{init, post_upgrade};
    use ic_oxigraph;

    thread_local! {
        // Feed the RNG with a seed of 32 bytes and pass this reference to the library.
        /* flexible */ static _CDK_RNG_REF_CELL: RefCell<StdRng> = RefCell::new(SeedableRng::from_seed([0_u8; 32]));
    }

    #[init]
    fn init() {
        _CDK_RNG_REF_CELL.with(ic_oxigraph::init);
        // other init code
    }

    #[post_upgrade]
    fn post_upgrade() {
        _CDK_RNG_REF_CELL.with(ic_oxigraph::init);
        // other post_upgrade code like loading the stable memory into the state
    }
    ```
    See [lib.rs](https://github.com/omnia-network/ic-oxigraph/blob/main/lib/src/lib.rs#L35-L71) for an example of how to register a custom getrandom implementation. See also Demergent Labs' [cdk_framework](https://github.com/demergent-labs/cdk_framework/blob/7e913d7ac49affad4a0bd5ee24b51b1a5d5d6096/src/act/random.rs) source code, from which this code has been taken.

- enable the `internal-rng` feature, which takes care of calling the management canister to get a random seed and register a custom getrandom. Example:
    ```rust
    use ic_cdk_macros::{init, post_upgrade};
    use ic_oxigraph;

    #[init]
    fn init() {
        ic_oxigraph::init();
        // other init code
    }

    #[post_upgrade]
    fn post_upgrade() {
        ic_oxigraph::init();
        // other post_upgrade code like loading the stable memory into the state
    }
    ```

> In both cases, the `init()` function **must** be called in the `init` and `post_upgrade` functions of the canister that imports this library.

### rust-analyzer
If you're using [rust-analyzer](https://rust-analyzer.github.io/) in VSCode, edit this setting to target `wasm32-unknown-unknown` and have the correct autocompletion:
```json
{
  "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
}
```

### TODO
As suggested in [this comment](https://github.com/oxigraph/oxigraph/issues/471#issuecomment-1544552518), this repository should have a script that automatically removes all the unnecessary code from original Oxigraph's repository, so that updates can be easily merged.

## Help

See [Oxigraph's GitHub discussions](https://github.com/oxigraph/oxigraph/discussions) or [the Oxigraph's Gitter chat](https://gitter.im/oxigraph/community) to ask questions or talk about Oxigraph.
If you want to report bugs, use [Oxigraph's Bug reports](https://github.com/oxigraph/oxigraph/issues).

## License

This project is licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).

### Contribution

This is an untested implementation that needs a lot of help from the community! Feel free to open PRs :).

