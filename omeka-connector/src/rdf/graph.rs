use std::borrow::Cow;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Iri(Cow<'static, str>);

impl Iri {
    pub fn new<T: Into<Cow<'static, str>>>(iri: T) -> Self {
        Self(iri.into())
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(Cow<'static, str>),
}

#[derive(Debug, Clone)]
pub enum Subject {
    Iri(Iri),
}

impl From<Iri> for Subject {
    fn from(iri: Iri) -> Self {
        Self::Iri(iri)
    }
}

#[derive(Debug, Clone)]
pub enum Predicate {
    Iri(Iri),
    Type,
}

impl From<Iri> for Predicate {
    fn from(iri: Iri) -> Self {
        Self::Iri(iri)
    }
}

#[derive(Debug, Clone)]
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
    const fn from_static(prefix: &'static str, ns: &'static str) -> Self {
        Self {
            prefix: Cow::Borrowed(prefix),
            ns: Cow::Borrowed(ns),
        }
    }

    pub fn get(&self, name: &str) -> Iri {
        Iri::new(format!("{}:{}", self.prefix, name))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Context {
    dcterms: Namespace,
    dctype: Namespace,
    schema: Namespace,
    xsd: Namespace,
    a: Namespace,
}

pub const CTX: Context = Context {
    dcterms: Namespace::from_static("dcterms", "http://purl.org/dc/terms/"),
    dctype: Namespace::from_static("dctype", "http://purl.org/dc/dcmitype/"),
    schema: Namespace::from_static("schema", "http://schema.org/"),
    xsd: Namespace::from_static("xsd", "http://www.w3.org/2001/XMLSchema#"),
    a: Namespace::from_static("a", "https://acearchive.lgbt/a/"),
};

#[derive(Debug, Clone, Default)]
pub struct Graph(Vec<(Subject, Predicate, Object)>);

impl Graph {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert<S, P, O>(&mut self, subject: S, predicate: P, object: O)
    where
        S: Into<Subject>,
        P: Into<Predicate>,
        O: Into<Object>,
    {
        self.0
            .push((subject.into(), predicate.into(), object.into()));
    }
}

pub fn build() -> Graph {
    let mut graph = Graph::new();

    const ID: &str = "MKbNWmmhCog6";

    graph.insert(CTX.a.get(ID), Predicate::Type, CTX.dctype.get("Text"));
    graph.insert(CTX.a.get(ID), CTX.dcterms.get("identifier"), ID);
    graph.insert(
        CTX.a.get(ID),
        CTX.schema.get("url"),
        "https://acearchive.lgbt/artifacts/orlando-the-asexual-manifesto",
    );
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("title"),
        "*The Asexual Manifesto*",
    );
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("description"),
        "A paper by the Asexual Caucus of the New York Radical Feminists",
    );
    graph.insert(CTX.a.get(ID), CTX.dcterms.get("created"), "1972");
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("subject"),
        Iri::new("https://homosaurus.org/v5/homoit0000004"),
    );
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("subject"),
        Iri::new("https://homosaurus.org/v5/homoit0000799"),
    );
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("relation"),
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
    );
    graph.insert(
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
        CTX.dcterms.get("title"),
        "Internet Archive",
    );
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("hasFormat"),
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        Predicate::Type,
        CTX.dctype.get("Text"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.dcterms.get("format"),
        "application/pdf",
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.dcterms.get("title"),
        "Digital Scan",
    );
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("isPartrOf"),
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press/"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press/"),
        Predicate::Type,
        CTX.dctype.get("Collection"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press/"),
        CTX.dcterms.get("title"),
        "News Journals and Underground Press",
    );
    graph.insert(
        CTX.a.get(ID),
        CTX.dcterms.get("creator"),
        Iri::new("https://acearchive.lgbt/people/lisa-orlando/"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando/"),
        Predicate::Type,
        CTX.dctype.get("Person"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando/"),
        CTX.dcterms.get("title"),
        "Lisa Orlando",
    );

    graph
}
