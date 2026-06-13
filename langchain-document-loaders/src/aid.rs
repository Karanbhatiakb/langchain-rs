/// Loader for aid data sources.
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use crate::traits::BaseLoader;

#[derive(Debug, Clone)]
pub struct AidLoader;

impl AidLoader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AidLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for AidLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        Ok(Vec::new())
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        futures::stream::empty().boxed()
    }
}
