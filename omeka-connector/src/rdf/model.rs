use std::fs;

use super::graph::{Context, Graph, Iri, Namespace, Predicate};
use crate::config;

#[derive(Debug, Clone)]
pub struct ModelContext {
    dcterms: Namespace,
    dctype: Namespace,
    schema: Namespace,
}

pub const CTX: ModelContext = ModelContext {
    dcterms: Namespace::from_static("dcterms", "http://purl.org/dc/terms/"),
    dctype: Namespace::from_static("dctype", "http://purl.org/dc/dcmitype/"),
    schema: Namespace::from_static("schema", "http://schema.org/"),
};

impl Context for ModelContext {
    fn namespaces(&self) -> Vec<&Namespace> {
        vec![&self.dcterms, &self.dctype, &self.schema]
    }
}

fn build() -> Graph<ModelContext> {
    let mut graph = Graph::new(CTX);

    const ARTIFACT_IRI: Iri = Iri::from_static("https://acearchive.lgbt/a/MKbNWmmhCog6");

    graph.insert(ARTIFACT_IRI, Predicate::Type, CTX.dctype.get("Text"));
    graph.insert(ARTIFACT_IRI, CTX.dcterms.get("identifier"), "MKbNWmmhCog6");
    graph.insert(
        ARTIFACT_IRI,
        CTX.schema.get("url"),
        "https://acearchive.lgbt/artifacts/orlando-the-asexual-manifesto",
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("title"),
        "*The Asexual Manifesto*",
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("description"),
        "A paper by the Asexual Caucus of the New York Radical Feminists",
    );
    graph.insert(ARTIFACT_IRI, CTX.dcterms.get("created"), "1972");
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("subject"),
        Iri::new("https://homosaurus.org/v5/homoit0000004"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("subject"),
        Iri::new("https://homosaurus.org/v5/homoit0000799"),
    );
    graph.insert(
        ARTIFACT_IRI,
        CTX.dcterms.get("relation"),
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
    );
    graph.insert(
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando"),
        CTX.dcterms.get("title"),
        "Internet Archive",
    );
    graph.insert(
        ARTIFACT_IRI,
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
        ARTIFACT_IRI,
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
        ARTIFACT_IRI,
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

pub fn generate() -> anyhow::Result<()> {
    let graph = build();

    let scratch_dir = config::scratch_dir()?;
    let jsonld_path = scratch_dir.join("archive.json");

    let file = fs::File::create(jsonld_path)?;
    serde_json::to_writer_pretty(file, &graph)?;

    Ok(())
}
