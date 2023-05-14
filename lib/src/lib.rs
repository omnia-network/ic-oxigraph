#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oxigraph/oxigraph/main/logo.svg")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oxigraph/oxigraph/main/logo.svg")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(test(attr(deny(warnings))))]
#![allow(clippy::return_self_not_must_use)]

pub mod io;
pub mod sparql;
mod storage;
pub mod store;

pub mod model {
    //! Implements data structures for [RDF 1.1 Concepts](https://www.w3.org/TR/rdf11-concepts/) using [OxRDF](https://crates.io/crates/oxrdf).

    pub use oxrdf::{
        dataset, graph, vocab, BlankNode, BlankNodeIdParseError, BlankNodeRef, Dataset, Graph,
        GraphName, GraphNameRef, IriParseError, LanguageTagParseError, Literal, LiteralRef,
        NamedNode, NamedNodeRef, NamedOrBlankNode, NamedOrBlankNodeRef, Quad, QuadRef, Subject,
        SubjectRef, Term, TermParseError, TermRef, Triple, TripleRef,
    };
}

use core::time::Duration;
use getrandom::{register_custom_getrandom, Error};
use ic_cdk::export::candid;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::cell::RefCell;

thread_local! {
    /* flexible */ static _CDK_RNG_REF_CELL: RefCell<StdRng> = RefCell::new(SeedableRng::from_seed([0_u8; 32]));
}

fn custom_getrandom(buf: &mut [u8]) -> Result<(), Error> {
    _CDK_RNG_REF_CELL.with(|rng_ref_cell| {
        let mut rng = rng_ref_cell.borrow_mut();
        rng.fill(buf);
    });

    Ok(())
}

fn rng_seed() {
    ic_cdk::spawn(async move {
        let result: ic_cdk::api::call::CallResult<(Vec<u8>,)> =
            ic_cdk::api::call::call(candid::Principal::management_canister(), "raw_rand", ()).await;

        _CDK_RNG_REF_CELL.with(|rng_ref_cell| {
            let mut rng = rng_ref_cell.borrow_mut();

            match result {
                Ok(randomness) => {
                    *rng = SeedableRng::from_seed(randomness.0[..].try_into().unwrap())
                }
                Err(err) => panic!("{:?}", err),
            };
        });
    });
}

register_custom_getrandom!(custom_getrandom);

/// Initializes the **Random Number Generator** by asynchronously calling the management canister to obtain a random seed.
///
/// This function **must** be called in the `init` and `post_upgrade` functions of the canister that imports this library.
/// 
/// # Example
/// ```rust
/// use ic_cdk_macros::{init, post_upgrade};
/// use ic_oxigraph;
///
/// #[init]
/// fn init() {
///     ic_oxigraph::init();
///     // other init code
/// }
/// 
/// #[post_upgrade]
/// fn post_upgrade() {
///     ic_oxigraph::init();
///     // other post_upgrade code like loading the stable memory into the state
/// }
/// ```
pub fn init(rng: Option<&RefCell<StdRng>>) {
    match rng {
        Some(rng) => _CDK_RNG_REF_CELL.with(|rng_ref_cell| {
            *rng_ref_cell.borrow_mut() = rng.borrow().clone();
        }),
        None => {
            ic_cdk_timers::set_timer(Duration::new(0, 0), rng_seed);
        },
    };
}
