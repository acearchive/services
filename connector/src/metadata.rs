/// An opaque type representing metadata associated with an item in the collection.
pub struct ItemMetadata(serde_json::Value);

/// A place where item metadata can be sent.
pub interface MetadataSink {
    pub fn send(&mut self, item: &ItemMetadata) -> anyhow::Result<()>;
}
