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

#[cfg(feature = "internal-rng")]
use core::time::Duration;
#[cfg(feature = "internal-rng")]
use getrandom::{register_custom_getrandom, Error};
#[cfg(feature = "internal-rng")]
use ic_cdk::export::candid;
#[cfg(feature = "internal-rng")]
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};
use std::cell::RefCell;

thread_local! {
    /* flexible */ static _CDK_RNG_REF_CELL: RefCell<StdRng> = RefCell::new(SeedableRng::from_seed([0_u8; 32]));
}

#[cfg(feature = "internal-rng")]
fn custom_getrandom(buf: &mut [u8]) -> Result<(), Error> {
    _CDK_RNG_REF_CELL.with(|rng_ref_cell| {
        let mut rng = rng_ref_cell.borrow_mut();
        rng.fill(buf);
    });

    Ok(())
}

#[cfg(feature = "internal-rng")]
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

#[cfg(feature = "internal-rng")]
register_custom_getrandom!(custom_getrandom);

/// Pass the **Random Number Generator** as a RefCell.
///
/// This function **must** be called in the `init` and `post_upgrade` functions of the canister that imports this library.
///
/// # Example
/// ```rust
/// use ic_cdk_macros::{init, post_upgrade};
/// use ic_oxigraph;
///
/// thread_local! {
///     // Feed the RNG with a seed of 32 bytes and pass this reference to the library.
///     /* flexible */ static _CDK_RNG_REF_CELL: RefCell<StdRng> = RefCell::new(SeedableRng::from_seed([0_u8; 32]));
/// }
/// 
/// #[init]
/// fn init() {
///     _CDK_RNG_REF_CELL.with(|rng_ref_cell| ic_oxigraph::init(Some(rng_ref_cell)));
///     // other init code
/// }
/// 
/// #[post_upgrade]
/// fn post_upgrade() {
///     _CDK_RNG_REF_CELL.with(|rng_ref_cell| ic_oxigraph::init(Some(rng_ref_cell)));
///     // other post_upgrade code like loading the stable memory into the state
/// }
/// ```
#[cfg(not(feature = "internal-rng"))]
pub fn init(rng: &RefCell<StdRng>) {
    _CDK_RNG_REF_CELL.with(|rng_ref_cell| {
        *rng_ref_cell.borrow_mut() = rng.borrow().clone();
    });
}

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
#[cfg(feature = "internal-rng")]
pub fn init() {
    ic_cdk_timers::set_timer(Duration::new(0, 0), rng_seed);
}
