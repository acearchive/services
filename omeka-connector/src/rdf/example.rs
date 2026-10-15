use std::fs;

use super::graph::{Context, Graph, Iri, Lang, Literal, Namespace, Predicate};
use crate::config;

#[derive(Debug, Clone)]
pub struct ModelContext {
    dcterms: Namespace,
    dctype: Namespace,
    schema: Namespace,
    rdfs: Namespace,
}

pub const CTX: ModelContext = ModelContext {
    dcterms: Namespace::from_static("dcterms", "http://purl.org/dc/terms/"),
    dctype: Namespace::from_static("dctype", "http://purl.org/dc/dcmitype/"),
    schema: Namespace::from_static("schema", "https://schema.org/"),
    rdfs: Namespace::from_static("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
};

pub const EN: Lang = Lang::from_static("en");

impl Context for ModelContext {
    fn namespaces(&self) -> Vec<&Namespace> {
        vec![&self.dcterms, &self.dctype, &self.schema, &self.rdfs]
    }
}

fn build() -> Graph<ModelContext> {
    let mut graph = Graph::new(CTX);

    const ARTIFACT_IRI: Iri = Iri::from_static("https://acearchive.lgbt/a/MKbNWmmhCog6#this");

    graph.insert(ARTIFACT_IRI, Predicate::Type, CTX.dctype.get("Text"));
    graph.insert(
        ARTIFACT_IRI,
        Predicate::Type,
        CTX.schema.get("CreativeWork"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("identifier"),
        Literal::from("MKbNWmmhCog6"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("identifier"),
        Literal::from("MKbNWmmhCog6"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("url"),
        Iri::new("https://acearchive.lgbt/artifacts/orlando-the-asexual-manifesto"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("title"),
        ("The Asexual Manifesto".into(), EN),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("name"),
        ("The Asexual Manifesto".into(), EN),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("description"),
        (
            "A paper by the Asexual Caucus of the New York Radical Feminists".into(),
            EN,
        ),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("description"),
        (
            "A paper by the Asexual Caucus of the New York Radical Feminists".into(),
            EN,
        ),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("abstract"),
        (
            "A paper by the Asexual Caucus of the New York Radical Feminists".into(),
            EN,
        ),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("abstract"),
        (
            "A paper by the Asexual Caucus of the New York Radical Feminists".into(),
            EN,
        ),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("created"),
        Literal::from("1972"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("dateCreated"),
        Literal::from("1972"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("subject"),
        Iri::new("https://homosaurus.org/v5/homoit0000004"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("about"),
        Iri::new("https://homosaurus.org/v5/homoit0000004"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("subject"),
        Iri::new("https://homosaurus.org/v5/homoit0000799"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("about"),
        Iri::new("https://homosaurus.org/v5/homoit0000799"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("relation"),
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.rdfs.get("seeAlso"),
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
    );
    graph.insert(
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
        CTX.dcterms.get("title"),
        ("Internet Archive".into(), EN),
    );
    graph.insert(
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
        CTX.schema.get("name"),
        ("Internet Archive".into(), EN),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("hasFormat"),
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("encoding"),
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.schema.get("contentUrl"),
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        Predicate::Type,
        CTX.dctype.get("Text"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        Predicate::Type,
        CTX.schema.get("MediaObject"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.dcterms.get("format"),
        Literal::from("application/pdf"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.schema.get("encodingFormat"),
        Literal::from("application/pdf"),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.dcterms.get("title"),
        ("Digital Scan".into(), EN),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.schema.get("name"),
        ("Digital Scan".into(), EN),
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.dcterms.get("isFormatOf"),
        ARTIFACT_IRI,
    );
    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf"),
        CTX.schema.get("encodesCreativeWork"),
        ARTIFACT_IRI,
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("isPartOf"),
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press#this"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("isPartOf"),
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press#this"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press#this"),
        Predicate::Type,
        CTX.dctype.get("Collection"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press#this"),
        Predicate::Type,
        CTX.schema.get("Collection"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press#this"),
        CTX.dcterms.get("title"),
        ("News Journals and Underground Press".into(), EN),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press#this"),
        CTX.schema.get("name"),
        ("News Journals and Underground Press".into(), EN),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press#this"),
        CTX.schema.get("url"),
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("creator"),
        Iri::new("https://acearchive.lgbt/people/lisa-orlando#this"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("creator"),
        Iri::new("https://acearchive.lgbt/people/lisa-orlando#this"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando#this"),
        Predicate::Type,
        CTX.dcterms.get("Agent"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando#this"),
        Predicate::Type,
        CTX.schema.get("Person"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando#this"),
        CTX.dcterms.get("title"),
        Literal::from("Lisa Orlando"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando#this"),
        CTX.schema.get("name"),
        Literal::from("Lisa Orlando"),
    );
    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando#this"),
        CTX.schema.get("url"),
        Iri::new("https://acearchive.lgbt/people/lisa-orlando"),
    );

    graph
}

pub fn generate() -> anyhow::Result<()> {
    let graph = build();

    let scratch_dir = config::scratch_dir()?;
    let jsonld_path = scratch_dir.join("archive.json");

    let file = fs::File::create(jsonld_path)?;
    serde_json::to_writer_pretty(file, &graph)?;

    Ok(())
}
