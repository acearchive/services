use std::{
    fs::{self, File},
    io::Write,
    sync::LazyLock,
};

use sophia::{
    api::{
        graph::{Graph, MutableGraph},
        ns::{Namespace, rdf},
        serializer::{QuadSerializer, Stringifier},
        source::TripleSource,
    },
    inmem::graph::LightGraph,
    iri::Iri,
    jsonld::{ContextRef, JsonLdOptions, JsonLdStringifier, context::TryIntoContextRef},
};

use crate::config;

#[derive(Debug, Clone)]
struct Namespaces {
    dcterms: Namespace<&'static str>,
    dctype: Namespace<&'static str>,
    schema: Namespace<&'static str>,
    a: Namespace<&'static str>,
}

impl Namespaces {
    fn context(&self) -> anyhow::Result<ContextRef> {
        serde_json::json!({
            "@context": {
                "dcterms": self.dcterms.as_str(),
                "dctype": self.dctype.as_str(),
                "schema": self.schema.as_str(),
                "a": self.a.as_str(),
            }
        })
        .to_string()
        .as_str()
        .try_into_context_ref()
        .map_err(|e| anyhow::anyhow!("Failed to create context: {}", e))
    }
}

static NS: LazyLock<Namespaces> = LazyLock::new(|| Namespaces {
    dcterms: Namespace::new("http://purl.org/dc/terms/").unwrap(),
    dctype: Namespace::new("http://purl.org/dc/dcmitype/").unwrap(),
    schema: Namespace::new("http://schema.org/").unwrap(),
    a: Namespace::new("https://acearchive.lgbt/a/").unwrap(),
});

fn build() -> anyhow::Result<LightGraph> {
    let mut graph = LightGraph::new();

    let id = "MKbNWmmhCog6";

    graph.insert(NS.a.get(id)?, rdf::type_, NS.dctype.get("Text")?)?;

    graph.insert(NS.a.get(id)?, NS.dcterms.get("identifier")?, id)?;

    graph.insert(
        NS.a.get(id)?,
        NS.schema.get("url")?,
        Iri::new("https://acearchive.lgbt/artifacts/orlando-the-asexual-manifesto")?,
    )?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("title")?,
        "*The Asexual Manifesto*",
    )?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("description")?,
        "A paper by the Asexual Caucus of the New York Radical Feminists",
    )?;

    graph.insert(NS.a.get(id)?, NS.dcterms.get("created")?, 1972)?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("subject")?,
        Iri::new("https://homosaurus.org/v5/homoit0000004")?,
    )?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("subject")?,
        Iri::new("https://homosaurus.org/v5/homoit0000799")?,
    )?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("relation")?,
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando")?,
    )?;

    graph.insert(
        Iri::new("https://archive.org/details/asexualmanifestolisaorlando")?,
        NS.dcterms.get("title")?,
        "Internet Archive",
    )?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("hasFormat")?,
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf")?,
    )?;

    graph.insert(
        Iri::new("https://files.acearchive.lgbt/r/MKbNWmmhCog6/the-asexual-manifesto.pdf")?,
        rdf::type_,
        NS.dctype.get("Text")?,
    )?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("isPartOf")?,
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press/")?,
    )?;

    graph.insert(
        Iri::new("https://acearchive.lgbt/collections/newsjournals-and-underground-press/")?,
        rdf::type_,
        NS.dctype.get("Collection")?,
    )?;

    graph.insert(
        NS.a.get(id)?,
        NS.dcterms.get("creator")?,
        Iri::new("https://acearchive.lgbt/people/lisa-orlando/")?,
    )?;

    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando/")?,
        rdf::type_,
        NS.dctype.get("Person")?,
    )?;

    graph.insert(
        Iri::new("https://acearchive.lgbt/people/lisa-orlando/")?,
        NS.dcterms.get("title")?,
        "Lisa Orlando",
    )?;

    Ok(graph)
}

fn serialize(graph: impl Graph) -> anyhow::Result<String> {
    let options = JsonLdOptions::new()
        .with_spaces(2)
        .with_compact_context(NS.context()?);
    let mut serializer = JsonLdStringifier::new_stringifier_with_options(options);
    serializer.serialize_quads(graph.triples().to_quads())?;
    Ok(serializer.to_string())
}

pub fn write() -> anyhow::Result<()> {
    let graph = build()?;
    let jsonld = serialize(&graph)?;

    let scratch_dir = config::scratch_dir()?;
    let out_path = scratch_dir.join("archive.json");
    fs::create_dir_all(scratch_dir)?;
    let mut out_file = File::create(&out_path)?;
    out_file.write_all(jsonld.as_bytes())?;

    Ok(())
}
