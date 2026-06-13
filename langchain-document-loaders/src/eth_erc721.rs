//! Loader for Ethereum ERC-721 NFT token data.
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use crate::traits::BaseLoader;

#[derive(Debug, Clone)]
pub struct EthErc721Loader;

impl EthErc721Loader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EthErc721Loader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for EthErc721Loader {
    async fn load(&self) -> Result<Vec<Document>> {
        Ok(Vec::new())
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        futures::stream::empty().boxed()
    }
}
