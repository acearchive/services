use super::graph::{Context, Graph, Iri, Lang, Literal, Namespace, Object, Predicate};
use crate::{models, omeka, url};

#[derive(Debug, Clone)]
struct Namespaces {
    dcterms: Namespace,
    dctype: Namespace,
    schema: Namespace,
    rdfs: Namespace,
    ace: Namespace,
}

const NS: Namespaces = Namespaces {
    dcterms: Namespace::from_static("dcterms", "http://purl.org/dc/terms/"),
    dctype: Namespace::from_static("dctype", "http://purl.org/dc/dcmitype/"),
    schema: Namespace::from_static("schema", "https://schema.org/"),
    rdfs: Namespace::from_static("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
    ace: Namespace::from_static("ace", "https://acearchive.lgbt/terms/"),
};

#[derive(Debug, Clone)]
struct ModelContext {
    ns: Namespaces,
    include_private_terms: bool,
}

impl Context for ModelContext {
    fn namespaces(&self) -> Vec<&Namespace> {
        let mut terms = vec![
            &self.ns.dcterms,
            &self.ns.dctype,
            &self.ns.schema,
            &self.ns.rdfs,
        ];

        if self.include_private_terms {
            terms.push(&self.ns.ace);
        }

        terms
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
    fn from_web_url(url: &reqwest::Url) -> Iri {
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

#[derive(Debug, Clone)]
pub struct ModelOptions {
    /// Include project-specific RDF terms.
    pub include_private_terms: bool,
}

#[derive(Debug)]
pub struct ModelSerializer {
    graph: Graph<ModelContext>,
    include_private_terms: bool,
}

impl ModelSerializer {
    pub fn new(options: ModelOptions) -> Self {
        Self {
            graph: Graph::new(ModelContext {
                ns: NS,
                include_private_terms: options.include_private_terms,
            }),
            include_private_terms: options.include_private_terms,
        }
    }
    fn add_link(&mut self, link: &models::Link) {
        let link_iri = Iri::new(link.url.to_string());

        self.graph.insert(
            link_iri.clone(),
            NS.dcterms.get("title"),
            link.title.clone(),
        );
        self.graph
            .insert(link_iri, NS.schema.get("name"), link.title.clone());
    }

    fn add_person(&mut self, person: &models::Person) -> anyhow::Result<()> {
        let person_url = url::person(&person.name)?;
        let person_iri = Iri::from_web_url(&person_url);

        self.graph
            .insert(person_iri.clone(), Predicate::Type, NS.dcterms.get("Agent"));
        self.graph
            .insert(person_iri.clone(), Predicate::Type, NS.schema.get("Person"));

        self.graph.insert(
            person_iri.clone(),
            NS.schema.get("url"),
            Iri::new(person_url.to_string()),
        );

        self.graph.insert(
            person_iri.clone(),
            NS.dcterms.get("title"),
            Literal::from(person.name.clone()),
        );
        self.graph.insert(
            person_iri,
            NS.schema.get("name"),
            Literal::from(person.name.clone()),
        );

        Ok(())
    }

    fn add_collection(&mut self, collection: &models::Collection) -> anyhow::Result<()> {
        let collection_url = url::collection(&collection.title.value)?;
        let collection_iri = Iri::from_web_url(&collection_url);

        self.graph.insert(
            collection_iri.clone(),
            Predicate::Type,
            NS.dctype.get("Collection"),
        );
        self.graph.insert(
            collection_iri.clone(),
            Predicate::Type,
            NS.schema.get("Collection"),
        );

        self.graph.insert(
            collection_iri.clone(),
            NS.schema.get("url"),
            Iri::new(collection_url.to_string()),
        );

        self.graph.insert(
            collection_iri.clone(),
            NS.dcterms.get("title"),
            collection.title.clone(),
        );
        self.graph.insert(
            collection_iri.clone(),
            NS.schema.get("name"),
            collection.title.clone(),
        );

        if let Some(description) = &collection.description {
            self.graph.insert(
                collection_iri.clone(),
                NS.dcterms.get("description"),
                description.clone(),
            );
            self.graph.insert(
                collection_iri,
                NS.schema.get("description"),
                description.clone(),
            );
        }

        Ok(())
    }

    fn add_file(&mut self, file: &models::File, id: &omeka::AceId) -> anyhow::Result<()> {
        let file_iri = Iri::new(url::file_raw(id, &file.filename)?.to_string());
        let item_iri = Iri::from_web_url(&url::artifact_short(id)?);

        if self.include_private_terms {
            self.graph.insert(
                file_iri.clone(),
                NS.ace.get("filename"),
                Literal::from(file.filename.to_string()),
            );
        }

        self.graph.insert(
            file_iri.clone(),
            NS.schema.get("contentUrl"),
            file_iri.clone(),
        );

        self.graph
            .insert(file_iri.clone(), Predicate::Type, NS.dctype.get("Text"));
        self.graph.insert(
            file_iri.clone(),
            Predicate::Type,
            NS.schema.get("MediaObject"),
        );

        self.graph.insert(
            file_iri.clone(),
            NS.dcterms.get("format"),
            Literal::from(file.media_type.clone()),
        );
        self.graph.insert(
            file_iri.clone(),
            NS.schema.get("encodingFormat"),
            Literal::from(file.media_type.clone()),
        );

        self.graph.insert(
            file_iri.clone(),
            NS.dcterms.get("title"),
            file.title.clone(),
        );
        self.graph
            .insert(file_iri.clone(), NS.schema.get("name"), file.title.clone());

        self.graph.insert(
            file_iri.clone(),
            NS.dcterms.get("isFormatOf"),
            item_iri.clone(),
        );
        self.graph.insert(
            file_iri.clone(),
            NS.schema.get("encodesCreativeWork"),
            item_iri,
        );

        Ok(())
    }

    pub fn add_item(&mut self, item: &models::Item) -> anyhow::Result<()> {
        let item_url = url::artifact_short(&item.id)?;
        let item_iri = Iri::from_web_url(&item_url);

        self.graph
            .insert(item_iri.clone(), Predicate::Type, NS.dctype.get("Text"));
        self.graph.insert(
            item_iri.clone(),
            Predicate::Type,
            NS.schema.get("CreativeWork"),
        );

        if self.include_private_terms {
            self.graph.insert(
                item_iri.clone(),
                NS.ace.get("slug"),
                Literal::from(item.slug.to_string()),
            );
        }

        self.graph.insert(
            item_iri.clone(),
            NS.dcterms.get("identifier"),
            Literal::from(item.id.to_string()),
        );
        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("identifier"),
            Literal::from(item.id.to_string()),
        );

        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("url"),
            Iri::new(item_url.to_string()),
        );

        self.graph.insert(
            item_iri.clone(),
            NS.dcterms.get("title"),
            item.title.clone(),
        );
        self.graph
            .insert(item_iri.clone(), NS.schema.get("name"), item.title.clone());

        if let Some(description) = &item.description {
            self.graph.insert(
                item_iri.clone(),
                NS.dcterms.get("description"),
                description.clone(),
            );
            self.graph.insert(
                item_iri.clone(),
                NS.schema.get("description"),
                description.clone(),
            );
        }

        self.graph.insert(
            item_iri.clone(),
            NS.dcterms.get("abstract"),
            item.summary.clone(),
        );
        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("abstract"),
            item.summary.clone(),
        );

        let year_range = if let Some(to_year) = item.to_year {
            format!("{}/{}", item.from_year, to_year)
        } else {
            item.from_year.to_string()
        };

        self.graph.insert(
            item_iri.clone(),
            NS.dcterms.get("created"),
            Literal::from(year_range.clone()),
        );
        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("dateCreated"),
            Literal::from(year_range),
        );

        for identity in &item.identities {
            let identity_object = match &identity.url {
                Some(identity_url) => Object::from(Iri::new(identity_url.to_string())),
                None => Object::from(identity.name.clone()),
            };

            self.graph.insert(
                item_iri.clone(),
                NS.dcterms.get("subject"),
                identity_object.clone(),
            );
            self.graph
                .insert(item_iri.clone(), NS.schema.get("about"), identity_object);
        }

        for link in &item.links {
            self.add_link(link);

            self.graph.insert(
                item_iri.clone(),
                NS.dcterms.get("relation"),
                Iri::new(link.url.to_string()),
            );
            self.graph.insert(
                item_iri.clone(),
                NS.rdfs.get("seeAlso"),
                Iri::new(link.url.to_string()),
            );
        }

        for file in &item.files {
            self.add_file(file, &item.id)?;

            let file_url = url::file_raw(&item.id, &file.filename)?;

            self.graph.insert(
                item_iri.clone(),
                NS.dcterms.get("hasFormat"),
                Iri::new(file_url.to_string()),
            );
            self.graph.insert(
                item_iri.clone(),
                NS.schema.get("encoding"),
                Iri::new(file_url.to_string()),
            )
        }

        for collection in &item.collections {
            self.add_collection(collection)?;

            let collection_url = url::collection(&collection.title.value)?;

            self.graph.insert(
                item_iri.clone(),
                NS.dcterms.get("isPartOf"),
                Iri::from_web_url(&collection_url),
            );
            self.graph.insert(
                item_iri.clone(),
                NS.schema.get("isPartOf"),
                Iri::from_web_url(&collection_url),
            );
        }

        for person in &item.people {
            self.add_person(person)?;

            let person_url = url::person(&person.name)?;

            self.graph.insert(
                item_iri.clone(),
                NS.dcterms.get("creator"),
                Iri::from_web_url(&person_url),
            );
            self.graph.insert(
                item_iri.clone(),
                NS.schema.get("creator"),
                Iri::from_web_url(&person_url),
            );
        }

        Ok(())
    }

    pub fn to_json(&self) -> serde_json::Value {
        self.graph.to_json()
    }
}
