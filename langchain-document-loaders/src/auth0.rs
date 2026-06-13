use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use crate::traits::BaseLoader;

#[derive(Debug, Clone)]
pub struct Uauth0Loader;

impl Uauth0Loader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Uauth0Loader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for Uauth0Loader {
    async fn load(&self) -> Result<Vec<Document>> {
        Ok(Vec::new())
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        futures::stream::empty().boxed()
    }
}
