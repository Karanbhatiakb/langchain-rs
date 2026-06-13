use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_document_loaders::traits::BaseLoader;
use langchain_document_loaders::text::TextLoader;
use langchain_document_loaders::csv::CSVLoader;
use langchain_document_loaders::json_loader::{JSONLoader, JSONLinesLoader};
use tokio::fs;

#[cfg(feature = "s3")]
use langchain_document_loaders::s3::S3Loader;
#[cfg(feature = "azure_blob")]
use langchain_document_loaders::azure_blob::AzureBlobLoader;
#[cfg(feature = "dropbox")]
use langchain_document_loaders::dropbox::DropboxLoader;
#[cfg(feature = "box_store")]
use langchain_document_loaders::box_store::BoxLoader;
#[cfg(feature = "upstage")]
use langchain_document_loaders::upstage::UpstageLoader;
#[cfg(feature = "docling")]
use langchain_document_loaders::docling::DoclingLoader;
#[cfg(feature = "pymupdf")]
use langchain_document_loaders::pymupdf::PyMuPDFLoader;

async fn create_temp_file(content: &str, suffix: &str) -> tempfile::NamedTempFile {
    let file = tempfile::Builder::new()
        .suffix(suffix)
        .rand_bytes(6)
        .tempfile()
        .unwrap();
    fs::write(file.path(), content).await.unwrap();
    file
}

#[tokio::test]
async fn test_text_loader_load() {
    let file = create_temp_file("Hello, world!", ".txt").await;
    let loader = TextLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].page_content, "Hello, world!");
    assert_eq!(docs[0].metadata.get("loader_type").unwrap(), "text");
}

#[tokio::test]
async fn test_text_loader_with_encoding() {
    let file = create_temp_file("Test content", ".txt").await;
    let loader = TextLoader::new(file.path().to_str().unwrap()).with_encoding("utf-8");
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 1);
}

#[tokio::test]
async fn test_text_loader_lazy_load() {
    let file = create_temp_file("Lazy content", ".txt").await;
    let loader = TextLoader::new(file.path().to_str().unwrap());
    let stream = loader.lazy_load().await;
    let docs: Vec<Document> = stream
        .filter_map(|d| async move { d.ok() })
        .collect()
        .await;
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].page_content, "Lazy content");
}

#[tokio::test]
async fn test_text_loader_metadata_source() {
    let file = create_temp_file("content", ".txt").await;
    let path_str = file.path().to_str().unwrap().to_string();
    let loader = TextLoader::new(&path_str);
    let docs = loader.load().await.unwrap();
    assert_eq!(
        docs[0].metadata.get("source").unwrap().as_str().unwrap(),
        path_str
    );
}

#[tokio::test]
async fn test_text_loader_nonexistent_file() {
    let loader = TextLoader::new("/nonexistent/path/file.txt");
    let result = loader.load().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_csv_loader_load() {
    let content = "name,age\nAlice,30\nBob,25\n";
    let file = create_temp_file(content, ".csv").await;
    let loader = CSVLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 2);
    let first: serde_json::Value = serde_json::from_str(&docs[0].page_content).unwrap();
    assert_eq!(first["name"], "Alice");
    assert_eq!(first["age"], "30");
}

#[tokio::test]
async fn test_csv_loader_with_delimiter() {
    let content = "name|age\nAlice|30\n";
    let file = create_temp_file(content, ".csv").await;
    let loader = CSVLoader::new(file.path().to_str().unwrap()).with_delimiter(b'|');
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 1);
}

#[tokio::test]
async fn test_csv_loader_metadata() {
    let content = "name,age\nAlice,30\n";
    let file = create_temp_file(content, ".csv").await;
    let loader = CSVLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs[0].metadata.get("loader_type").unwrap(), "csv");
    assert_eq!(docs[0].metadata.get("row").unwrap(), 0);
}

#[tokio::test]
async fn test_csv_loader_lazy_load() {
    let content = "name,age\nAlice,30\nBob,25\n";
    let file = create_temp_file(content, ".csv").await;
    let loader = CSVLoader::new(file.path().to_str().unwrap());
    let stream = loader.lazy_load().await;
    let docs: Vec<Document> = stream
        .filter_map(|d| async move { d.ok() })
        .collect()
        .await;
    assert_eq!(docs.len(), 2);
}

#[tokio::test]
async fn test_json_loader_array() {
    let content = r#"[{"name": "Alice"}, {"name": "Bob"}]"#;
    let file = create_temp_file(content, ".json").await;
    let loader = JSONLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 2);
}

#[tokio::test]
async fn test_json_loader_object() {
    let content = r#"{"name": "Alice", "age": 30}"#;
    let file = create_temp_file(content, ".json").await;
    let loader = JSONLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 1);
}

#[tokio::test]
async fn test_json_loader_with_jq_schema() {
    let content = r#"{"data": [{"id": 1}, {"id": 2}]}"#;
    let file = create_temp_file(content, ".json").await;
    let loader = JSONLoader::new(file.path().to_str().unwrap()).with_jq_schema(".data");
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 2);
}

#[tokio::test]
async fn test_json_loader_metadata() {
    let content = r#"[{"name": "Alice"}]"#;
    let file = create_temp_file(content, ".json").await;
    let loader = JSONLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs[0].metadata.get("loader_type").unwrap(), "json");
}

#[tokio::test]
async fn test_json_loader_nonexistent_file() {
    let loader = JSONLoader::new("/nonexistent/file.json");
    let result = loader.load().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_json_loader_invalid_json() {
    let file = create_temp_file("not valid json {{{", ".json").await;
    let loader = JSONLoader::new(file.path().to_str().unwrap());
    let result = loader.load().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jsonl_loader() {
    let content = "{\"name\": \"Alice\"}\n{\"name\": \"Bob\"}\n";
    let file = create_temp_file(&content, ".jsonl").await;
    let loader = JSONLinesLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 2);
}

#[tokio::test]
async fn test_jsonl_loader_skips_empty_lines() {
    let content = "{\"name\": \"Alice\"}\n\n{\"name\": \"Bob\"}\n";
    let file = create_temp_file(&content, ".jsonl").await;
    let loader = JSONLinesLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs.len(), 2);
}

#[tokio::test]
async fn test_jsonl_loader_metadata() {
    let content = "{\"name\": \"Alice\"}\n";
    let file = create_temp_file(&content, ".jsonl").await;
    let loader = JSONLinesLoader::new(file.path().to_str().unwrap());
    let docs = loader.load().await.unwrap();
    assert_eq!(docs[0].metadata.get("loader_type").unwrap(), "jsonl");
    assert_eq!(docs[0].metadata.get("line").unwrap(), 0);
}

#[cfg(feature = "s3")]
mod s3_tests {
    use super::*;
    #[test]
    fn test_s3_loader_creation() {
        let loader = S3Loader::new("my-bucket", "us-east-1", "path/file.txt", "access", "secret");
        let _ = &loader;
    }
}

#[cfg(feature = "azure_blob")]
mod azure_blob_tests {
    use super::*;
    #[test]
    fn test_azure_blob_loader_creation() {
        let loader = AzureBlobLoader::new("myaccount", "container", "blob.txt", "sas-token");
        let _ = &loader;
    }
}

#[cfg(feature = "dropbox")]
mod dropbox_tests {
    use super::*;
    #[test]
    fn test_dropbox_loader_creation() {
        let loader = DropboxLoader::new("access-token", "/path/to/file.txt");
        let _ = &loader;
    }
}

#[cfg(feature = "box_store")]
mod box_store_tests {
    use super::*;
    #[test]
    fn test_box_loader_creation() {
        let loader = BoxLoader::new("access-token", "file-id-123");
        let _ = &loader;
    }
}

#[cfg(feature = "upstage")]
mod upstage_tests {
    use super::*;
    #[test]
    fn test_upstage_loader_creation() {
        let loader = UpstageLoader::new("test-key", "document-data");
        let _ = &loader;
    }
    #[test]
    fn test_upstage_loader_with_output_format() {
        let loader = UpstageLoader::new("test-key", "document-data")
            .with_output_format("html");
        let _ = &loader;
    }
}

#[cfg(feature = "docling")]
mod docling_tests {
    use super::*;
    #[test]
    fn test_docling_loader_creation() {
        let loader = DoclingLoader::new("document-data");
        let _ = &loader;
    }
    #[test]
    fn test_docling_loader_with_endpoint() {
        let loader = DoclingLoader::new("document-data")
            .with_endpoint("http://custom:5001");
        let _ = &loader;
    }
}

#[cfg(feature = "pymupdf")]
mod pymupdf_tests {
    use super::*;
    #[test]
    fn test_pymupdf_loader_creation() {
        let loader = PyMuPDFLoader::new("/path/to/file.pdf");
        let _ = &loader;
    }
}
