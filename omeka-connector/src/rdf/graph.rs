use std::{borrow::Cow, fmt};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Iri(Cow<'static, str>);

impl Iri {
    pub fn new<T: Into<Cow<'static, str>>>(iri: T) -> Self {
        Self(iri.into())
    }
}

impl fmt::Display for Iri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    String(Cow<'static, str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subject {
    Iri(Iri),
}

impl From<Iri> for Subject {
    fn from(iri: Iri) -> Self {
        Self::Iri(iri)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Predicate {
    Iri(Iri),
    Type,
}

impl From<Iri> for Predicate {
    fn from(iri: Iri) -> Self {
        Self::Iri(iri)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Object {
    Iri(Iri),
    Literal(Literal),
}

impl From<Iri> for Object {
    fn from(iri: Iri) -> Self {
        Self::Iri(iri)
    }
}

impl From<String> for Object {
    fn from(s: String) -> Self {
        Self::Literal(Literal::String(Cow::Owned(s)))
    }
}

impl From<&'static str> for Object {
    fn from(s: &'static str) -> Self {
        Self::Literal(Literal::String(Cow::Borrowed(s)))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Namespace {
    #[serde(skip)]
    prefix: Cow<'static, str>,
    ns: Cow<'static, str>,
}

impl Namespace {
    pub const fn from_static(prefix: &'static str, ns: &'static str) -> Self {
        Self {
            prefix: Cow::Borrowed(prefix),
            ns: Cow::Borrowed(ns),
        }
    }

    pub fn get(&self, name: &str) -> Iri {
        Iri::new(format!("{}:{}", self.prefix, name))
    }
}

pub trait Context {
    fn namespaces(&self) -> Vec<&Namespace>;
}

#[derive(Debug, Clone)]
pub(super) struct Triple {
    pub s: Subject,
    pub p: Predicate,
    pub o: Object,
}

#[derive(Debug, Clone)]
pub struct Graph<C> {
    pub(super) context: C,
    pub(super) triples: Vec<Triple>,
}

impl<C> Graph<C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            triples: Vec::new(),
        }
    }

    pub fn insert<S, P, O>(&mut self, subject: S, predicate: P, object: O)
    where
        S: Into<Subject>,
        P: Into<Predicate>,
        O: Into<Object>,
    {
        self.triples.push(Triple {
            s: subject.into(),
            p: predicate.into(),
            o: object.into(),
        });
    }
}
