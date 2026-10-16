use strip_markdown::strip_markdown;

use super::graph::{Context, Graph, Iri, Lang, Literal, Namespace, Object, Predicate};
use crate::{models, omeka, url};

#[derive(Debug, Clone)]
struct Namespaces {
    schema: Namespace,
    rdfs: Namespace,
    ace: Namespace,
}

const NS: Namespaces = Namespaces {
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
        let mut terms = vec![&self.ns.schema, &self.ns.rdfs];

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

        self.graph
            .insert(link_iri.clone(), Predicate::Type, NS.schema.get("WebPage"));

        self.graph
            .insert(link_iri, NS.schema.get("name"), link.title.clone());
    }

    fn add_person(&mut self, person: &models::Person) -> anyhow::Result<()> {
        let person_url = url::person(&person.name)?;
        let person_iri = Iri::from_web_url(&person_url);

        self.graph
            .insert(person_iri.clone(), Predicate::Type, NS.schema.get("Person"));

        self.graph.insert(
            person_iri.clone(),
            NS.schema.get("url"),
            Iri::new(person_url.to_string()),
        );

        self.graph.insert(
            person_iri.clone(),
            NS.schema.get("name"),
            Literal::from(person.name.clone()),
        );

        Ok(())
    }

    fn add_identity_scheme(&mut self) -> anyhow::Result<()> {
        let scheme_iri = Iri::from_web_url(&url::identities()?);

        self.graph
            .insert(scheme_iri, Predicate::Type, NS.schema.get("DefinedTermSet"));

        Ok(())
    }

    fn add_identity(&mut self, identity: &models::Identity) -> anyhow::Result<()> {
        let identity_iri = Iri::from_web_url(&url::identity(&identity.name.value)?);
        let scheme_iri = Iri::from_web_url(&url::identities()?);

        self.graph.insert(
            identity_iri.clone(),
            Predicate::Type,
            NS.schema.get("DefinedTerm"),
        );

        self.graph.insert(
            identity_iri.clone(),
            NS.schema.get("name"),
            identity.name.clone(),
        );

        if let Some(identity_url) = &identity.url {
            self.graph.insert(
                identity_iri.clone(),
                NS.schema.get("sameAs"),
                Iri::new(identity_url.to_string()),
            );
        }

        self.graph.insert(
            identity_iri.clone(),
            NS.schema.get("inDefinedTermSet"),
            scheme_iri.clone(),
        );

        self.add_identity_scheme()?;

        Ok(())
    }

    fn add_collection(&mut self, collection: &models::Collection) -> anyhow::Result<()> {
        let collection_url = url::collection(&collection.title.value)?;
        let collection_iri = Iri::from_web_url(&collection_url);

        if let Some(schema_class) = &collection.schema_class {
            self.graph.insert(
                collection_iri.clone(),
                Predicate::Type,
                NS.schema.get(schema_class.local_name()),
            );
        }

        self.graph.insert(
            collection_iri.clone(),
            NS.schema.get("url"),
            Iri::new(collection_url.to_string()),
        );

        self.graph.insert(
            collection_iri.clone(),
            NS.schema.get("name"),
            collection.title.clone(),
        );

        if let Some(description) = &collection.description {
            self.graph.insert(
                collection_iri.clone(),
                NS.schema.get("description"),
                description.clone(),
            );
        }

        Ok(())
    }

    pub fn add_file(&mut self, file: &models::File, id: &omeka::AceId) -> anyhow::Result<()> {
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

        if let Some(schema_class) = &file.schema_class {
            self.graph.insert(
                file_iri.clone(),
                Predicate::Type,
                NS.schema.get(schema_class.local_name()),
            );
        }

        self.graph.insert(
            file_iri.clone(),
            NS.schema.get("encodingFormat"),
            Literal::from(file.media_type.clone()),
        );

        self.graph
            .insert(file_iri.clone(), NS.schema.get("name"), file.title.clone());

        self.graph.insert(
            file_iri.clone(),
            NS.schema.get("encodesCreativeWork"),
            item_iri.clone(),
        );

        Ok(())
    }

    pub fn add_item(&mut self, item: &models::Item) -> anyhow::Result<()> {
        let item_url = url::artifact_short(&item.id)?;
        let item_iri = Iri::from_web_url(&item_url);

        if let Some(schema_class) = &item.schema_class {
            self.graph.insert(
                item_iri.clone(),
                Predicate::Type,
                NS.schema.get(schema_class.local_name()),
            );
        }

        if self.include_private_terms {
            self.graph.insert(
                item_iri.clone(),
                NS.ace.get("slug"),
                Literal::from(item.slug.to_string()),
            );
        }

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

        let stripped_title = models::LocalizedString {
            value: strip_markdown(&item.title.value),
            lang: item.title.lang.clone(),
        };

        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("name"),
            stripped_title.clone(),
        );

        let stripped_abstract = models::LocalizedString {
            value: strip_markdown(&item.summary.value),
            lang: item.summary.lang.clone(),
        };

        if let Some(description) = &item.description
            && description != &item.summary
        {
            let stripped_description = models::LocalizedString {
                value: strip_markdown(&description.value),
                lang: description.lang.clone(),
            };

            self.graph.insert(
                item_iri.clone(),
                NS.schema.get("description"),
                stripped_description.clone(),
            );

            self.graph.insert(
                item_iri.clone(),
                NS.schema.get("abstract"),
                stripped_abstract.clone(),
            );
        } else {
            self.graph.insert(
                item_iri.clone(),
                NS.schema.get("description"),
                stripped_abstract.clone(),
            );
        }

        // The `schema::dateCreated` term only supports individual years, not ranges.
        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("dateCreated"),
            Literal::from(item.from_year.to_string()),
        );

        for identity in &item.identities {
            self.add_identity(identity)?;
            self.link_about(&item_iri, identity)?;
        }

        for link in &item.links {
            self.add_link(link);

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
                NS.schema.get("encoding"),
                Iri::new(file_url.to_string()),
            );
        }

        for collection in &item.collections {
            self.add_collection(collection)?;
            self.link_part_of(&item_iri, collection)?;
        }

        for person in &item.people {
            self.add_person(person)?;
            self.link_creator(&item_iri, person)?;
        }

        Ok(())
    }

    fn item_iri(item: &models::Item) -> anyhow::Result<Iri> {
        Ok(Iri::from_web_url(&url::artifact_short(&item.id)?))
    }

    fn link_about(&mut self, item_iri: &Iri, identity: &models::Identity) -> anyhow::Result<()> {
        let identity_url = url::identity(&identity.name.value)?;

        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("about"),
            Iri::from_web_url(&identity_url),
        );

        Ok(())
    }

    fn link_part_of(
        &mut self,
        item_iri: &Iri,
        collection: &models::Collection,
    ) -> anyhow::Result<()> {
        let collection_url = url::collection(&collection.title.value)?;

        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("isPartOf"),
            Iri::from_web_url(&collection_url),
        );

        Ok(())
    }

    fn link_creator(&mut self, item_iri: &Iri, person: &models::Person) -> anyhow::Result<()> {
        let person_url = url::person(&person.name)?;

        self.graph.insert(
            item_iri.clone(),
            NS.schema.get("creator"),
            Iri::from_web_url(&person_url),
        );

        Ok(())
    }

    /// Add a person, along with a link from each item they created.
    pub fn add_person_document(
        &mut self,
        person: &models::Person,
        items: &[models::Item],
    ) -> anyhow::Result<()> {
        self.add_person(person)?;

        for item in items {
            self.link_creator(&Self::item_iri(item)?, person)?;
        }

        Ok(())
    }

    /// Add an identity, along with a link from each item about it.
    pub fn add_identity_document(
        &mut self,
        identity: &models::Identity,
        items: &[models::Item],
    ) -> anyhow::Result<()> {
        self.add_identity(identity)?;

        for item in items {
            self.link_about(&Self::item_iri(item)?, identity)?;
        }

        Ok(())
    }

    /// Add a collection, along with a link from each item in it.
    pub fn add_collection_document(
        &mut self,
        collection: &models::Collection,
        items: &[models::Item],
    ) -> anyhow::Result<()> {
        self.add_collection(collection)?;

        for item in items {
            self.link_part_of(&Self::item_iri(item)?, collection)?;
        }

        Ok(())
    }

    pub fn into_json(self) -> anyhow::Result<serde_json::Value> {
        Ok(self.graph.to_json())
    }
}
