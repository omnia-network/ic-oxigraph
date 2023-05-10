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

use core::num::NonZeroU32;
use getrandom::{register_custom_getrandom, Error};
use rand::Rng;
use rand_chacha::{
    rand_core::SeedableRng,
    ChaCha20Rng,
};

// a custom error code not well thought out
const CUSTOM_ERROR_CODE: u32 = Error::CUSTOM_START + 1;

fn getrandom_from_timestamp(buf: &mut [u8]) -> Result<(), Error> {
    let timestamp = ic_cdk::api::time();
    let mut rng = ChaCha20Rng::seed_from_u64(timestamp);

    rng.try_fill(buf).map_err(|err| {
        if let Some(code) = err.code() {
            Error::from(code)
        } else {
            let code = NonZeroU32::new(CUSTOM_ERROR_CODE).unwrap();
            Error::from(code)
        }
    })
}

register_custom_getrandom!(getrandom_from_timestamp);
