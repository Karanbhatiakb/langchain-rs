use langchain_tools::traits::BaseTool;
use std::sync::Arc;

pub trait BaseToolkit: Send + Sync {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>>;
    fn name(&self) -> &str;
}
