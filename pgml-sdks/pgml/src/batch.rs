//! Upsert documents in batches.

#[cfg(feature = "rust_bridge")]
use rust_bridge::{alias, alias_methods};

use tracing::instrument;

use crate::{types::Json, Collection};

#[cfg(feature = "python")]
use crate::{collection::CollectionPython, types::JsonPython};

#[cfg(feature = "c")]
use crate::{collection::CollectionC, languages::c::JsonC};

/// A batch of documents staged for upsert
#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[derive(Debug, Clone)]
pub struct Batch {
    collection: Collection,
    pub(crate) documents: Vec<Json>,
    pub(crate) size: i64,
    pub(crate) args: Option<Json>,
}

#[cfg_attr(feature = "rust_bridge", alias_methods(new, upsert_documents, finish,))]
impl Batch {
    /// Create a new upsert batch.
    ///
    /// # Arguments
    ///
    /// * `collection` - The collection to upsert documents to.
    /// * `size` - The size of the batch.
    /// * `args` - Optional arguments to pass to the upsert operation.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Collection, Batch};
    ///
    /// let collection = Collection::new("my_collection");
    /// let batch = Batch::new(&collection, 100, None);
    /// ```
    pub fn new(collection: &Collection, size: i64, args: Option<Json>) -> Self {
        Self {
            collection: collection.clone(),
            args,
            documents: Vec::new(),
            size,
        }
    }

    /// Upsert documents into the collection. If the batch is full, save the documents.
    ///
    /// When using this method, remember to call [finish](Batch::finish) to save any remaining documents
    /// in the last batch.
    ///
    /// # Arguments
    ///
    /// * `documents` - The documents to upsert.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Collection, Batch};
    /// use serde_json::json;
    ///
    /// let collection = Collection::new("my_collection");
    /// let mut batch = Batch::new(&collection, 100, None);
    ///
    /// batch.upsert_documents(vec![json!({"id": 1}), json!({"id": 2})]).await?;
    /// batch.finish().await?;
    /// ```
    #[instrument(skip(self))]
    pub async fn upsert_documents(&mut self, documents: Vec<Json>) -> anyhow::Result<()> {
        for document in documents {
            if self.size as usize >= self.documents.len() {
                self.collection
                    .upsert_documents(std::mem::take(&mut self.documents), self.args.clone())
                    .await?;
                self.documents.clear();
            }

            self.documents.push(document);
        }

        Ok(())
    }

    /// Save any remaining documents in the last batch.
    #[instrument(skip(self))]
    pub async fn finish(&mut self) -> anyhow::Result<()> {
        if !self.documents.is_empty() {
            self.collection
                .upsert_documents(std::mem::take(&mut self.documents), self.args.clone())
                .await?;
        }

        Ok(())
    }
}
