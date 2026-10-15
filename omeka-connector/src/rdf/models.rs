use super::graph::{Context, Graph, Iri, Lang, Literal, Namespace, Object, Predicate};
use crate::{models, omeka, url};

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

impl<C> Graph<C> {
    fn add_link(&mut self, link: &models::Link) {
        let link_iri = Iri::new(link.url.to_string());

        self.insert(
            link_iri.clone(),
            CTX.dcterms.get("title"),
            link.title.clone(),
        );
        self.insert(link_iri, CTX.schema.get("name"), link.title.clone());
    }

    fn add_file(&mut self, file: &models::File, id: &omeka::AceId) -> anyhow::Result<()> {
        let file_iri = Iri::new(url::file_raw(id, &file.filename)?.to_string());
        let item_iri = Iri::from_web_url(&url::artifact_short(id)?);

        self.insert(
            file_iri.clone(),
            CTX.schema.get("contentUrl"),
            file_iri.clone(),
        );

        self.insert(file_iri.clone(), Predicate::Type, CTX.dctype.get("Text"));
        self.insert(
            file_iri.clone(),
            Predicate::Type,
            CTX.schema.get("MediaObject"),
        );

        self.insert(
            file_iri.clone(),
            CTX.dcterms.get("format"),
            Literal::from(file.media_type.clone()),
        );
        self.insert(
            file_iri.clone(),
            CTX.schema.get("encodingFormat"),
            Literal::from(file.media_type.clone()),
        );

        self.insert(
            file_iri.clone(),
            CTX.dcterms.get("title"),
            file.title.clone(),
        );
        self.insert(file_iri.clone(), CTX.schema.get("name"), file.title.clone());

        self.insert(
            file_iri.clone(),
            CTX.dcterms.get("isFormatOf"),
            item_iri.clone(),
        );
        self.insert(
            file_iri.clone(),
            CTX.schema.get("encodesCreativeWork"),
            item_iri,
        );

        Ok(())
    }

    fn add_collection(&mut self, collection: &models::Collection) -> anyhow::Result<()> {
        let collection_url = url::collection(&collection.title.value)?;
        let collection_iri = Iri::from_web_url(&collection_url);

        self.insert(
            collection_iri.clone(),
            Predicate::Type,
            CTX.dctype.get("Collection"),
        );
        self.insert(
            collection_iri.clone(),
            Predicate::Type,
            CTX.schema.get("Collection"),
        );

        self.insert(
            collection_iri.clone(),
            CTX.schema.get("url"),
            Literal::from(collection_url.clone().to_string()),
        );

        self.insert(
            collection_iri.clone(),
            CTX.dcterms.get("title"),
            collection.title.clone(),
        );
        self.insert(
            collection_iri.clone(),
            CTX.schema.get("name"),
            collection.title.clone(),
        );

        if let Some(description) = &collection.description {
            self.insert(
                collection_iri.clone(),
                CTX.dcterms.get("description"),
                description.clone(),
            );
            self.insert(
                collection_iri,
                CTX.schema.get("description"),
                description.clone(),
            );
        }

        Ok(())
    }
}
