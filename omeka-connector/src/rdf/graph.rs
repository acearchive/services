use std::{borrow::Cow, collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Iri(Cow<'static, str>);

impl Iri {
    pub fn new<T: Into<Cow<'static, str>>>(iri: T) -> Self {
        Self(iri.into())
    }

    pub const fn from_static(iri: &'static str) -> Self {
        Self(Cow::Borrowed(iri))
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

impl From<String> for Literal {
    fn from(s: String) -> Self {
        Self::String(Cow::Owned(s))
    }
}

impl From<&'static str> for Literal {
    fn from(s: &'static str) -> Self {
        Self::String(Cow::Borrowed(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lang(Cow<'static, str>);

impl Lang {
    pub fn new<T: Into<Cow<'static, str>>>(lang: T) -> Self {
        Self(lang.into())
    }

    pub const fn from_static(lang: &'static str) -> Self {
        Self(Cow::Borrowed(lang))
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
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
    Literal(Literal, Option<Lang>),
}

impl From<Iri> for Object {
    fn from(iri: Iri) -> Self {
        Self::Iri(iri)
    }
}

impl From<Literal> for Object {
    fn from(literal: Literal) -> Self {
        Self::Literal(literal, None)
    }
}

impl From<(Literal, Lang)> for Object {
    fn from((literal, lang): (Literal, Lang)) -> Self {
        Self::Literal(literal, Some(lang))
    }
}

#[derive(Debug, Clone)]
pub struct Namespace {
    pub prefix: Cow<'static, str>,
    pub namespace: Cow<'static, str>,
}

impl Namespace {
    pub const fn from_static(prefix: &'static str, ns: &'static str) -> Self {
        Self {
            prefix: Cow::Borrowed(prefix),
            namespace: Cow::Borrowed(ns),
        }
    }

    pub fn get<S: AsRef<str>>(&self, name: S) -> Iri {
        Iri::new(format!("{}:{}", self.prefix, name.as_ref()))
    }
}

pub trait Context {
    fn namespaces(&self) -> Vec<&Namespace>;

    fn types(&self) -> HashMap<Iri, Iri> {
        Default::default()
    }
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
