use super::graph::{Context, Graph, Iri, Lang, Literal, Namespace, Object, ToGraph};
use crate::models;

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

impl Context for ModelContext {
    fn namespaces(&self) -> Vec<&Namespace> {
        vec![&self.dcterms, &self.dctype, &self.schema, &self.rdfs]
    }
}

impl Iri {
    // When converting a URL to a web page to an IRI, we append a fragment to distinguish the
    // *entity* the URL represents from the *web page* it points to. For example, if we're
    // referencing a URL that represents a creative work, the date the work was created is different
    // from the date the web page was created. So, we need to distinguish them in the RDF graph.
    // Appending a fragment like `#this` is a common convention for doing this.
    //
    // We only do this with web URLs that represent semantic nodes in the RDF graph--not just any
    // web URL.
    //
    // This guidance comes from the W3C:
    // https://www.w3.org/TR/cooluris/
    pub fn from_web_url(url: &reqwest::Url) -> Iri {
        Iri::new(format!("{}#this", url.to_string().trim_end_matches('/')))
    }
}

impl From<models::Lang> for Lang {
    fn from(lang: models::Lang) -> Self {
        Self::new(lang.to_string())
    }
}

impl From<models::LocalizedString> for Object {
    fn from(ls: models::LocalizedString) -> Self {
        Self::Literal(Literal::from(ls.value), ls.lang.map(Lang::from))
    }
}

impl<C> ToGraph<C> for models::Link {
    fn to_graph(&self, graph: &mut Graph<C>) {
        let iri = Iri::new(self.url.to_string());

        graph.insert(iri.clone(), CTX.dcterms.get("title"), self.title.clone());
        graph.insert(iri, CTX.schema.get("name"), self.title.clone());
    }
}
