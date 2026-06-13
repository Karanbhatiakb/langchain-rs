//! Tool implementations for interacting with external systems and APIs.
//!
//! Provides the [`BaseTool`](traits::BaseTool) trait in [`traits`] and
//! implementations for calculation, shell, Python, HTTP requests, file system,
//! databases, weather, Wikipedia, ArXiv, PubMed, Wolfram Alpha, GitHub, Notion,
//! Slack, Discord, Gmail, Google Calendar, and many more — each gated behind a
//! feature flag.

pub mod traits;
pub mod calculator;
pub mod shell;
pub mod python;
pub mod requests;
pub mod file_system;
pub mod database;
pub mod weather;
pub mod wikipedia;
pub mod arxiv;
pub mod pubmed;
#[cfg(feature = "wolfram_alpha")]
pub mod wolfram_alpha;
pub mod github;
pub mod notion;
pub mod slack;
pub mod discord;
pub mod gmail;
pub mod google_calendar;
pub mod search;
pub mod coding;
pub mod comms;
pub mod data;
pub mod office;
pub mod utility;
pub mod openweathermap;
pub mod ddg_search;
pub mod searx_search;
pub mod searchapi;
pub mod jira_tool;
pub mod gitlab_tool;
pub mod azure_cognitive;
pub mod e2b_tool;
pub mod edenai;
pub mod eleven_labs;
pub mod nasa;
pub mod openapi_tool;
pub mod powerbi;
pub mod reddit_search;
pub mod sql_database_tool;
pub mod golden_query;
pub mod google_jobs;
pub mod google_lens;
pub mod google_places;
pub mod merriam_webster;
pub mod metaphor_search;
pub mod multion;
pub mod nuclia;
pub mod office365;
#[cfg(feature = "sleep_tool")]
pub mod sleep_tool;
pub mod steamship;
#[allow(non_snake_case)]
pub mod sceneXplain_tool;
pub mod human_tool;
pub mod youtube_tool;
pub mod twitter_tool;
pub mod spark_sql;
pub mod stackexchange;
pub mod steam_tool;
pub mod yahoo_finance;
pub mod ainetwork;
pub mod amadeus;
pub mod bearly;
pub mod clickup;
pub mod dataforseo;
pub mod edenai_multi;
pub mod google_cloud;
pub mod google_finance;
pub mod google_scholar;
pub mod google_trends;
pub mod ifttt;
#[cfg(feature = "tavily")]
pub mod tavily;
#[cfg(feature = "google_search")]
pub mod google_search;
#[cfg(feature = "bing_search")]
pub mod bing_search;
#[cfg(feature = "brave_search")]
pub mod brave_search;
#[cfg(feature = "serpapi")]
pub mod serpapi;
#[cfg(feature = "exa_search")]
pub mod exa_search;
#[cfg(feature = "you_search")]
pub mod you_search;
#[cfg(feature = "graphql")]
pub mod graphql;
#[cfg(feature = "dalle")]
pub mod dalle;
#[cfg(feature = "playwright")]
pub mod playwright;
#[cfg(feature = "zapier")]
pub mod zapier;
#[cfg(feature = "polygon")]
pub mod polygon;
#[cfg(feature = "scenexplain")]
pub mod scenexplain;
#[cfg(feature = "spotify")]
pub mod spotify;
#[cfg(feature = "salesforce")]
pub mod salesforce;
#[cfg(feature = "sharepoint")]
pub mod sharepoint;
#[cfg(feature = "zendesk")]
pub mod zendesk;
#[cfg(feature = "hubspot")]
pub mod hubspot;
#[cfg(feature = "vectorstore_qa")]
pub mod vectorstore_qa;
#[cfg(feature = "google_serper")]
pub mod google_serper;
#[cfg(feature = "ai_plugin")]
pub mod ai_plugin;
#[cfg(feature = "alpha_vantage")]
pub mod alpha_vantage;
#[cfg(feature = "json_get_value")]
pub mod json_get_value;
#[cfg(feature = "json_list_keys")]
pub mod json_list_keys;
#[cfg(feature = "twilio")]
pub mod twilio;
#[cfg(feature = "requests_get")]
pub mod requests_get;
#[cfg(feature = "requests_post")]
pub mod requests_post;
#[cfg(feature = "bash_shell")]
pub mod bash_shell;
#[cfg(feature = "python_repl")]
pub mod python_repl;

pub mod providers;

#[cfg(test)]
mod tests {
    use super::traits::BaseTool;
    use async_trait::async_trait;

    struct TestTool;

    #[async_trait]
    impl BaseTool for TestTool {
        fn name(&self) -> &str { "test" }
        fn description(&self) -> &str { "A test tool" }
        async fn invoke(&self, input: &str) -> super::traits::ToolResult {
            Ok(format!("processed: {}", input))
        }
    }

    #[tokio::test]
    async fn test_tool_construct_and_invoke() {
        let tool = TestTool;
        let result = tool.invoke("data").await.unwrap();
        assert_eq!(result, "processed: data");
    }

    #[tokio::test]
    async fn test_tool_trait_methods() {
        let tool = TestTool;
        assert_eq!(tool.name(), "test");
        assert_eq!(tool.description(), "A test tool");
    }

    #[tokio::test]
    async fn test_tool_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TestTool>();
        assert_sync::<TestTool>();
    }
}
