use crate::sparql::algebra::QueryDataset;
use crate::sparql::EvaluationError;
use crate::store::numeric_encoder::{
    EncodedQuad, EncodedTerm, ReadEncoder, StrContainer, StrEncodingAware, StrHash, StrLookup,
};
use crate::store::ReadableEncodedStore;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::{empty, once, Once};

pub(crate) struct DatasetView<S: ReadableEncodedStore> {
    store: S,
    extra: RefCell<HashMap<StrHash, String>>,
    dataset: EncodedDatasetSpec,
}

impl<S: ReadableEncodedStore> DatasetView<S> {
    pub fn new(store: S, dataset: &QueryDataset) -> Result<Self, EvaluationError> {
        let dataset = EncodedDatasetSpec {
            default: dataset
                .default_graph_graphs()
                .map(|graphs| {
                    graphs
                        .iter()
                        .flat_map(|g| store.get_encoded_graph_name(g.as_ref()).transpose())
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()
                .map_err(|e| e.into())?,
            named: dataset
                .available_named_graphs()
                .map(|graphs| {
                    graphs
                        .iter()
                        .flat_map(|g| {
                            store
                                .get_encoded_named_or_blank_node(g.as_ref())
                                .transpose()
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()
                .map_err(|e| e.into())?,
        };
        Ok(Self {
            store,
            extra: RefCell::new(HashMap::default()),
            dataset,
        })
    }

    fn store_encoded_quads_for_pattern(
        &self,
        subject: Option<EncodedTerm>,
        predicate: Option<EncodedTerm>,
        object: Option<EncodedTerm>,
        graph_name: Option<EncodedTerm>,
    ) -> impl Iterator<Item = Result<EncodedQuad, EvaluationError>> + 'static {
        self.store
            .encoded_quads_for_pattern(subject, predicate, object, graph_name)
            .map(|t| t.map_err(|e| e.into()))
    }
}

impl<S: ReadableEncodedStore> StrEncodingAware for DatasetView<S> {
    type Error = EvaluationError;
}

impl<S: ReadableEncodedStore> StrLookup for DatasetView<S> {
    fn get_str(&self, id: StrHash) -> Result<Option<String>, EvaluationError> {
        self.extra
            .borrow()
            .get(&id)
            .cloned()
            .map(Ok)
            .or_else(|| self.store.get_str(id).map_err(|e| e.into()).transpose())
            .transpose()
    }

    fn get_str_id(&self, value: &str) -> Result<Option<StrHash>, EvaluationError> {
        let id = StrHash::new(value);
        Ok(if self.extra.borrow().contains_key(&id) {
            Some(id)
        } else {
            self.store.get_str_id(value).map_err(|e| e.into())?
        })
    }
}

impl<S: ReadableEncodedStore + 'static> ReadableEncodedStore for DatasetView<S> {
    type QuadsIter = Box<dyn Iterator<Item = Result<EncodedQuad, EvaluationError>>>;
    type GraphsIter = Once<Result<EncodedTerm, EvaluationError>>;

    #[allow(clippy::needless_collect)]
    fn encoded_quads_for_pattern(
        &self,
        subject: Option<EncodedTerm>,
        predicate: Option<EncodedTerm>,
        object: Option<EncodedTerm>,
        graph_name: Option<EncodedTerm>,
    ) -> Box<dyn Iterator<Item = Result<EncodedQuad, EvaluationError>>> {
        if let Some(graph_name) = graph_name {
            if graph_name.is_default_graph() {
                if let Some(default_graph_graphs) = &self.dataset.default {
                    if default_graph_graphs.len() == 1 {
                        // Single graph optimization
                        Box::new(
                            self.store_encoded_quads_for_pattern(
                                subject,
                                predicate,
                                object,
                                Some(default_graph_graphs[0]),
                            )
                            .map(|quad| {
                                let quad = quad?;
                                Ok(EncodedQuad::new(
                                    quad.subject,
                                    quad.predicate,
                                    quad.object,
                                    EncodedTerm::DefaultGraph,
                                ))
                            }),
                        )
                    } else {
                        let iters = default_graph_graphs
                            .iter()
                            .map(|graph_name| {
                                self.store_encoded_quads_for_pattern(
                                    subject,
                                    predicate,
                                    object,
                                    Some(*graph_name),
                                )
                            })
                            .collect::<Vec<_>>();
                        Box::new(iters.into_iter().flatten().map(|quad| {
                            let quad = quad?;
                            Ok(EncodedQuad::new(
                                quad.subject,
                                quad.predicate,
                                quad.object,
                                EncodedTerm::DefaultGraph,
                            ))
                        }))
                    }
                } else {
                    Box::new(self.store_encoded_quads_for_pattern(subject, predicate, object, None))
                }
            } else if self
                .dataset
                .named
                .as_ref()
                .map_or(true, |d| d.contains(&graph_name))
            {
                Box::new(self.store_encoded_quads_for_pattern(
                    subject,
                    predicate,
                    object,
                    Some(graph_name),
                ))
            } else {
                Box::new(empty())
            }
        } else if let Some(named_graphs) = &self.dataset.named {
            let iters = named_graphs
                .iter()
                .map(|graph_name| {
                    self.store_encoded_quads_for_pattern(
                        subject,
                        predicate,
                        object,
                        Some(*graph_name),
                    )
                })
                .collect::<Vec<_>>();
            Box::new(iters.into_iter().flatten())
        } else {
            Box::new(
                self.store_encoded_quads_for_pattern(subject, predicate, object, None)
                    .filter(|quad| match quad {
                        Err(_) => true,
                        Ok(quad) => quad.graph_name != EncodedTerm::DefaultGraph,
                    }),
            )
        }
    }

    fn encoded_named_graphs(&self) -> Self::GraphsIter {
        once(Err(EvaluationError::msg(
            "Graphs lookup is not implemented by DatasetView",
        )))
    }

    fn contains_encoded_named_graph(&self, _: EncodedTerm) -> Result<bool, EvaluationError> {
        Err(EvaluationError::msg(
            "Graphs lookup is not implemented by DatasetView",
        ))
    }
}

impl<'a, S: ReadableEncodedStore> StrContainer for &'a DatasetView<S> {
    fn insert_str(&self, value: &str) -> Result<StrHash, EvaluationError> {
        if let Some(hash) = self.store.get_str_id(value).map_err(|e| e.into())? {
            Ok(hash)
        } else {
            let hash = StrHash::new(value);
            self.extra
                .borrow_mut()
                .entry(hash)
                .or_insert_with(|| value.to_owned());
            Ok(hash)
        }
    }
}

struct EncodedDatasetSpec {
    default: Option<Vec<EncodedTerm>>,
    named: Option<Vec<EncodedTerm>>,
}
