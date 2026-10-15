use std::collections::HashMap;

use super::graph::{Graph, Object, Predicate, Subject, Triple};

impl<C> Graph<C> {
    fn by_subject(&self) -> HashMap<&Subject, Vec<&Triple>> {
        let mut map = HashMap::new();

        for triple in &self.triples {
            map.entry(&triple.s).or_insert_with(Vec::new).push(triple);
        }
        map
    }
}
