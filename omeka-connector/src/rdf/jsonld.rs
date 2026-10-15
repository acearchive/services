use std::collections::HashMap;

use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};

use super::graph::{Context, Graph, Literal, Object, Predicate, Subject};

struct ByPredicate<'a> {
    predicate: &'a Predicate,
    objects: Vec<&'a Object>,
}

struct BySubject<'a> {
    subject: &'a Subject,
    predicates: Vec<ByPredicate<'a>>,
}

impl Object {
    fn serialize(&self) -> JsonValue {
        match self {
            Object::Iri(iri) => json!({ "@id": iri.to_string() }),
            Object::Literal(Literal::String(string)) => JsonValue::String(string.to_string()),
        }
    }
}

impl BySubject<'_> {
    fn serialize(&self) -> JsonValue {
        let mut map = JsonMap::new();

        let Subject::Iri(iri) = self.subject;
        map.insert(String::from("@id"), JsonValue::String(iri.to_string()));

        for ByPredicate { predicate, objects } in &self.predicates {
            let key = match predicate {
                Predicate::Type => String::from("@type"),
                Predicate::Iri(iri) => iri.to_string(),
            };

            match objects.as_slice() {
                [object] => {
                    map.insert(key, object.serialize());
                }
                objects => {
                    map.insert(key, objects.iter().map(|o| o.serialize()).collect());
                }
            }
        }

        serde_json::Value::Object(map)
    }
}

impl<C> Graph<C> {
    fn by_subject(&self) -> Vec<BySubject<'_>> {
        let mut predicates_by_subject = HashMap::new();
        let mut objects_by_subject_predicate = HashMap::new();

        for triple in &self.triples {
            predicates_by_subject
                .entry(&triple.s)
                .or_insert_with(Vec::new)
                .push(&triple.p);

            objects_by_subject_predicate
                .entry((&triple.s, &triple.p))
                .or_insert_with(Vec::new)
                .push(&triple.o);
        }

        predicates_by_subject
            .into_iter()
            .map(|(subject, predicates)| BySubject {
                subject,
                predicates: predicates
                    .into_iter()
                    .map(|predicate| ByPredicate {
                        predicate,
                        objects: objects_by_subject_predicate
                            .get(&(subject, predicate))
                            .unwrap_or(&Vec::new())
                            .clone(),
                    })
                    .collect(),
            })
            .collect()
    }
}

impl<C> Graph<C>
where
    C: Context,
{
    fn serialize(&self) -> JsonValue {
        let subjects = self.by_subject();
        let json_subjects: Vec<JsonValue> = subjects.iter().map(|s| s.serialize()).collect();

        let mut context_map = JsonMap::new();

        for namespace in self.context.namespaces() {
            context_map.insert(
                namespace.prefix.to_string(),
                JsonValue::String(namespace.namespace.to_string()),
            );
        }

        for (predicate_iri, type_iri) in self.context.types() {
            context_map.insert(
                predicate_iri.to_string(),
                json!({ "@type": type_iri.to_string() }),
            );
        }

        let mut graph_map = JsonMap::new();
        graph_map.insert("@context".to_string(), JsonValue::Object(context_map));
        graph_map.insert("@graph".to_string(), JsonValue::Array(json_subjects));

        JsonValue::Object(graph_map)
    }
}

impl<C> Serialize for Graph<C>
where
    C: Context,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let json_value = self.serialize();
        json_value.serialize(serializer)
    }
}
