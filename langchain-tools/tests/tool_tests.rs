use langchain_tools::calculator::CalculatorTool;
use langchain_tools::shell::ShellTool;
use langchain_tools::python::PythonREPLTool;
use langchain_tools::traits::BaseTool;
use langchain_tools::weather::OpenWeatherMapTool;
use langchain_tools::file_system::{ReadFileTool, WriteFileTool, ListDirectoryTool, DeleteFileTool, CopyFileTool, MoveFileTool, FileSearchTool};
use langchain_tools::requests::{RequestsGetTool, RequestsPostTool, RequestsDeleteTool, RequestsPutTool, RequestsPatchTool};
use langchain_tools::database::{SQLDatabaseTool, ListTablesTool, QuerySQLTool};
use langchain_tools::wikipedia::WikipediaTool;
use langchain_tools::arxiv::ArxivTool;
use langchain_tools::pubmed::PubMedTool;
use langchain_tools::wolfram_alpha::WolframAlphaTool;
use langchain_tools::github::GitHubTool;
use langchain_tools::notion::NotionTool;
use langchain_tools::slack::SlackTool;
use langchain_tools::discord::DiscordTool;
use langchain_tools::gmail::GmailTool;
use langchain_tools::google_calendar::GoogleCalendarTool;
use langchain_tools::search::DuckDuckGoSearchTool;
use langchain_tools::coding::StackOverflowTool;
use langchain_tools::data::JSONAPITool;
use langchain_tools::comms::EmailTool;

#[cfg(feature = "tavily")]
use langchain_tools::tavily::TavilySearchTool;
#[cfg(feature = "google_search")]
use langchain_tools::google_search::GoogleSearchTool;
#[cfg(feature = "bing_search")]
use langchain_tools::bing_search::BingSearchTool;
#[cfg(feature = "brave_search")]
use langchain_tools::brave_search::BraveSearchTool;
#[cfg(feature = "serpapi")]
use langchain_tools::serpapi::SerpApiTool;
#[cfg(feature = "exa_search")]
use langchain_tools::exa_search::ExaSearchTool;
#[cfg(feature = "you_search")]
use langchain_tools::you_search::YouSearchTool;
#[cfg(feature = "graphql")]
use langchain_tools::graphql::GraphQLTool;
#[cfg(feature = "dalle")]
use langchain_tools::dalle::DalleTool;
#[cfg(feature = "playwright")]
use langchain_tools::playwright::PlaywrightTool;
#[cfg(feature = "zapier")]
use langchain_tools::zapier::ZapierNlaTool;
#[cfg(feature = "polygon")]
use langchain_tools::polygon::PolygonTool;
#[cfg(feature = "scenexplain")]
use langchain_tools::scenexplain::SceneXplainTool;
#[cfg(feature = "spotify")]
use langchain_tools::spotify::SpotifyTool;
#[cfg(feature = "salesforce")]
use langchain_tools::salesforce::SalesforceTool;
#[cfg(feature = "sharepoint")]
use langchain_tools::sharepoint::SharePointTool;
#[cfg(feature = "zendesk")]
use langchain_tools::zendesk::ZendeskTool;
#[cfg(feature = "hubspot")]
use langchain_tools::hubspot::HubSpotTool;

#[test]
fn test_calculator_basic() {
    let calc = CalculatorTool;
    assert_eq!(calc.name(), "calculator");
    assert!(!calc.description().is_empty());
}

#[tokio::test]
async fn test_calculator_addition() {
    let calc = CalculatorTool;
    let result = calc.invoke("2+3").await.unwrap();
    assert_eq!(result, "5");
}

#[tokio::test]
async fn test_calculator_multiplication() {
    let calc = CalculatorTool;
    let result = calc.invoke("3*4").await.unwrap();
    assert_eq!(result, "12");
}

#[tokio::test]
async fn test_calculator_division() {
    let calc = CalculatorTool;
    let result = calc.invoke("10/2").await.unwrap();
    assert_eq!(result, "5");
}

#[tokio::test]
async fn test_calculator_subtraction() {
    let calc = CalculatorTool;
    let result = calc.invoke("10-3").await.unwrap();
    assert_eq!(result, "7");
}

#[tokio::test]
async fn test_calculator_parentheses() {
    let calc = CalculatorTool;
    let result = calc.invoke("(2+3)*4").await.unwrap();
    assert_eq!(result, "20");
}

#[tokio::test]
async fn test_calculator_power() {
    let calc = CalculatorTool;
    let result = calc.invoke("2^3").await.unwrap();
    assert_eq!(result, "8");
}

#[tokio::test]
async fn test_calculator_sqrt() {
    let calc = CalculatorTool;
    let result = calc.invoke("sqrt(16)").await.unwrap();
    assert_eq!(result, "4");
}

#[tokio::test]
async fn test_calculator_pi() {
    let calc = CalculatorTool;
    let result = calc.invoke("pi").await.unwrap();
    let val: f64 = result.parse().unwrap();
    assert!((val - std::f64::consts::PI).abs() < 1e-10);
}

#[tokio::test]
async fn test_calculator_division_by_zero() {
    let calc = CalculatorTool;
    let result = calc.invoke("1/0").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_calculator_empty() {
    let calc = CalculatorTool;
    let result = calc.invoke("").await;
    assert!(result.is_err());
}

#[test]
fn test_shell_tool() {
    let shell = ShellTool::new();
    assert_eq!(shell.name(), "shell");
}

#[test]
fn test_python_repl_tool() {
    let py = PythonREPLTool::new();
    assert_eq!(py.name(), "python_repl");
}

#[test]
fn test_weather_tool() {
    let weather = OpenWeatherMapTool::new("test_key");
    assert_eq!(weather.name(), "open_weather_map");
}

#[test]
fn test_file_system_tools() {
    assert_eq!(ReadFileTool.name(), "read_file");
    assert_eq!(WriteFileTool.name(), "write_file");
    assert_eq!(ListDirectoryTool.name(), "list_directory");
    assert_eq!(CopyFileTool.name(), "copy_file");
    assert_eq!(MoveFileTool.name(), "move_file");
    assert_eq!(FileSearchTool.name(), "file_search");
    assert_eq!(DeleteFileTool.name(), "delete_file");
}

#[test]
fn test_requests_tools() {
    assert_eq!(RequestsGetTool::new().name(), "requests_get");
    assert_eq!(RequestsPostTool::new().name(), "requests_post");
    assert_eq!(RequestsPutTool::new().name(), "requests_put");
    assert_eq!(RequestsPatchTool::new().name(), "requests_patch");
    assert_eq!(RequestsDeleteTool::new().name(), "requests_delete");
}

#[test]
fn test_database_tools() {
    assert_eq!(SQLDatabaseTool::new("sqlite://test.db").name(), "sql_database");
    assert_eq!(ListTablesTool::new("sqlite://test.db").name(), "list_tables");
    assert_eq!(QuerySQLTool::new("sqlite://test.db").name(), "query_sql");
}

#[test]
fn test_search_tools() {
    assert_eq!(WikipediaTool::new().name(), "wikipedia");
    assert_eq!(ArxivTool::new().name(), "arxiv");
    assert_eq!(PubMedTool::new().name(), "pubmed");
    assert_eq!(WolframAlphaTool::new("test_key").name(), "wolfram_alpha");
    assert_eq!(DuckDuckGoSearchTool::new().name(), "duckduckgo_search");
}

#[test]
fn test_coding_tools() {
    assert_eq!(StackOverflowTool::new().name(), "stackoverflow");
}

#[test]
fn test_office_tools() {
    assert_eq!(GitHubTool::new("test_token").name(), "github");
    assert_eq!(NotionTool::new("test_key").name(), "notion");
    assert_eq!(SlackTool::new("test_token").name(), "slack");
    assert_eq!(DiscordTool::new("test_token").name(), "discord");
    assert_eq!(GmailTool::new("test_key").name(), "gmail");
    assert_eq!(GoogleCalendarTool::new("test_key").name(), "google_calendar");
}

#[test]
fn test_comm_tools() {
    assert_eq!(EmailTool::new("smtp.test.com", 587).name(), "email");
}

#[test]
fn test_data_tools() {
    assert_eq!(JSONAPITool::new("https://api.example.com").name(), "json_api");
}

#[tokio::test]
async fn test_read_file_not_found() {
    let tool = ReadFileTool;
    let result = tool.invoke("/nonexistent/path/file.txt").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_write_and_read_file() {
    let dir = std::env::temp_dir().join("langchain_rs_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_write.txt");
    let path_str = path.to_string_lossy().to_string();

    let write_tool = WriteFileTool;
    let content = "Hello from langchain-rs!";
    let input = format!("{}\n{}", path_str, content);
    write_tool.invoke(&input).await.unwrap();

    let read_tool = ReadFileTool;
    let result = read_tool.invoke(&path_str).await.unwrap();
    assert_eq!(result, content);

    std::fs::remove_file(&path).ok();
}

#[tokio::test]
async fn test_list_directory() {
    let tool = ListDirectoryTool;
    let dir = std::env::temp_dir().to_string_lossy().to_string();
    let result = tool.invoke(&dir).await;
    assert!(result.is_ok());
}

#[test]
fn test_tool_definition() {
    let calc = CalculatorTool;
    let def = calc.to_definition();
    assert_eq!(def.name, "calculator");
    assert!(!def.description.is_empty());
}

#[tokio::test]
async fn test_calculator_negative() {
    let calc = CalculatorTool;
    let result = calc.invoke("-5+3").await.unwrap();
    assert_eq!(result, "-2");
}

#[tokio::test]
async fn test_calculator_decimal() {
    let calc = CalculatorTool;
    let result = calc.invoke("0.5*10").await.unwrap();
    let val: f64 = result.parse().unwrap();
    assert!((val - 5.0).abs() < 1e-10);
}

#[tokio::test]
async fn test_calculator_complex() {
    let calc = CalculatorTool;
    let result = calc.invoke("2+3*4").await.unwrap();
    assert_eq!(result, "14");
}

#[tokio::test]
async fn test_calculator_sin() {
    let calc = CalculatorTool;
    let result = calc.invoke("sin(0)").await.unwrap();
    let val: f64 = result.parse().unwrap();
    assert!(val.abs() < 1e-10);
}

#[tokio::test]
async fn test_calculator_log() {
    let calc = CalculatorTool;
    let result = calc.invoke("log(100)").await.unwrap();
    let val: f64 = result.parse().unwrap();
    assert!((val - 2.0).abs() < 1e-10);
}

#[cfg(feature = "tavily")]
mod tavily_tests {
    use super::*;
    #[test]
    fn test_tavily_tool_creation() {
        let tool = TavilySearchTool::new();
        assert_eq!(tool.name(), "tavily_search");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_tavily_tool_with_api_key() {
        let tool = TavilySearchTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "tavily_search");
    }
}

#[cfg(feature = "google_search")]
mod google_search_tests {
    use super::*;
    #[test]
    fn test_google_search_tool_creation() {
        let tool = GoogleSearchTool::new();
        assert_eq!(tool.name(), "google_search");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_google_search_tool_with_api_key() {
        let tool = GoogleSearchTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "google_search");
    }
}

#[cfg(feature = "bing_search")]
mod bing_search_tests {
    use super::*;
    #[test]
    fn test_bing_search_tool_creation() {
        let tool = BingSearchTool::new();
        assert_eq!(tool.name(), "bing_search");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_bing_search_tool_with_api_key() {
        let tool = BingSearchTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "bing_search");
    }
}

#[cfg(feature = "brave_search")]
mod brave_search_tests {
    use super::*;
    #[test]
    fn test_brave_search_tool_creation() {
        let tool = BraveSearchTool::new();
        assert_eq!(tool.name(), "brave_search");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_brave_search_tool_with_api_key() {
        let tool = BraveSearchTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "brave_search");
    }
}

#[cfg(feature = "serpapi")]
mod serpapi_tests {
    use super::*;
    #[test]
    fn test_serpapi_tool_creation() {
        let tool = SerpApiTool::new();
        assert_eq!(tool.name(), "serpapi");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_serpapi_tool_with_api_key() {
        let tool = SerpApiTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "serpapi");
    }
}

#[cfg(feature = "exa_search")]
mod exa_search_tests {
    use super::*;
    #[test]
    fn test_exa_search_tool_creation() {
        let tool = ExaSearchTool::new();
        assert_eq!(tool.name(), "exa_search");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_exa_search_tool_with_api_key() {
        let tool = ExaSearchTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "exa_search");
    }
    #[test]
    fn test_exa_search_tool_with_base_url() {
        let tool = ExaSearchTool::new().with_base_url("https://custom.exa.ai");
        assert_eq!(tool.name(), "exa_search");
    }
}

#[cfg(feature = "you_search")]
mod you_search_tests {
    use super::*;
    #[test]
    fn test_you_search_tool_creation() {
        let tool = YouSearchTool::new();
        assert_eq!(tool.name(), "you_search");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_you_search_tool_with_api_key() {
        let tool = YouSearchTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "you_search");
    }
}

#[cfg(feature = "graphql")]
mod graphql_tests {
    use super::*;
    #[test]
    fn test_graphql_tool_creation() {
        let tool = GraphQLTool::new();
        assert_eq!(tool.name(), "graphql");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_graphql_tool_with_endpoint() {
        let tool = GraphQLTool::new().with_endpoint("https://api.example.com/graphql");
        assert_eq!(tool.name(), "graphql");
    }
    #[test]
    fn test_graphql_tool_with_header() {
        let tool = GraphQLTool::new().with_header("X-Custom", "value");
        assert_eq!(tool.name(), "graphql");
    }
    #[test]
    fn test_graphql_tool_with_api_key() {
        let tool = GraphQLTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "graphql");
    }
}

#[cfg(feature = "dalle")]
mod dalle_tests {
    use super::*;
    #[test]
    fn test_dalle_tool_creation() {
        let tool = DalleTool::new();
        assert_eq!(tool.name(), "dalle");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_dalle_tool_with_api_key() {
        let tool = DalleTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "dalle");
    }
    #[test]
    fn test_dalle_tool_with_model() {
        let tool = DalleTool::new().with_model("dall-e-3");
        assert_eq!(tool.name(), "dalle");
    }
    #[test]
    fn test_dalle_tool_with_size() {
        let tool = DalleTool::new().with_size("1024x1024");
        assert_eq!(tool.name(), "dalle");
    }
}

#[cfg(feature = "playwright")]
mod playwright_tests {
    use super::*;
    #[test]
    fn test_playwright_tool_creation() {
        let tool = PlaywrightTool::new();
        assert_eq!(tool.name(), "playwright");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_playwright_tool_with_base_url() {
        let tool = PlaywrightTool::new().with_base_url("http://localhost:3000");
        assert_eq!(tool.name(), "playwright");
    }
}

#[cfg(feature = "zapier")]
mod zapier_tests {
    use super::*;
    #[test]
    fn test_zapier_tool_creation() {
        let tool = ZapierNlaTool::new();
        assert_eq!(tool.name(), "zapier_nla");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_zapier_tool_with_api_key() {
        let tool = ZapierNlaTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "zapier_nla");
    }
}

#[cfg(feature = "polygon")]
mod polygon_tests {
    use super::*;
    #[test]
    fn test_polygon_tool_creation() {
        let tool = PolygonTool::new();
        assert_eq!(tool.name(), "polygon");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_polygon_tool_with_api_key() {
        let tool = PolygonTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "polygon");
    }
}

#[cfg(feature = "scenexplain")]
mod scenexplain_tests {
    use super::*;
    #[test]
    fn test_scenexplain_tool_creation() {
        let tool = SceneXplainTool::new();
        assert_eq!(tool.name(), "scenexplain");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_scenexplain_tool_with_api_key() {
        let tool = SceneXplainTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "scenexplain");
    }
}

#[cfg(feature = "spotify")]
mod spotify_tests {
    use super::*;
    #[test]
    fn test_spotify_tool_creation() {
        let tool = SpotifyTool::new();
        assert_eq!(tool.name(), "spotify");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_spotify_tool_with_access_token() {
        let tool = SpotifyTool::new().with_access_token("test-token");
        assert_eq!(tool.name(), "spotify");
    }
}

#[cfg(feature = "salesforce")]
mod salesforce_tests {
    use super::*;
    #[test]
    fn test_salesforce_tool_creation() {
        let tool = SalesforceTool::new();
        assert_eq!(tool.name(), "salesforce");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_salesforce_tool_with_access_token() {
        let tool = SalesforceTool::new().with_access_token("test-token");
        assert_eq!(tool.name(), "salesforce");
    }
    #[test]
    fn test_salesforce_tool_with_instance_url() {
        let tool = SalesforceTool::new().with_instance_url("https://instance.salesforce.com");
        assert_eq!(tool.name(), "salesforce");
    }
}

#[cfg(feature = "sharepoint")]
mod sharepoint_tests {
    use super::*;
    #[test]
    fn test_sharepoint_tool_creation() {
        let tool = SharePointTool::new();
        assert_eq!(tool.name(), "sharepoint");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_sharepoint_tool_with_access_token() {
        let tool = SharePointTool::new().with_access_token("test-token");
        assert_eq!(tool.name(), "sharepoint");
    }
    #[test]
    fn test_sharepoint_tool_with_site_id() {
        let tool = SharePointTool::new().with_site_id("site-1");
        assert_eq!(tool.name(), "sharepoint");
    }
}

#[cfg(feature = "zendesk")]
mod zendesk_tests {
    use super::*;
    #[test]
    fn test_zendesk_tool_creation() {
        let tool = ZendeskTool::new();
        assert_eq!(tool.name(), "zendesk");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_zendesk_tool_with_api_key() {
        let tool = ZendeskTool::new().with_api_key("test-key");
        assert_eq!(tool.name(), "zendesk");
    }
    #[test]
    fn test_zendesk_tool_with_email() {
        let tool = ZendeskTool::new().with_email("test@example.com");
        assert_eq!(tool.name(), "zendesk");
    }
    #[test]
    fn test_zendesk_tool_with_base_url() {
        let tool = ZendeskTool::new().with_base_url("https://mysubdomain.zendesk.com/api/v2");
        assert_eq!(tool.name(), "zendesk");
    }
}

#[cfg(feature = "hubspot")]
mod hubspot_tests {
    use super::*;
    #[test]
    fn test_hubspot_tool_creation() {
        let tool = HubSpotTool::new();
        assert_eq!(tool.name(), "hubspot");
        assert!(!tool.description().is_empty());
    }
    #[test]
    fn test_hubspot_tool_with_access_token() {
        let tool = HubSpotTool::new().with_access_token("test-token");
        assert_eq!(tool.name(), "hubspot");
    }
}

#[cfg(all(feature = "tavily", feature = "google_search", feature = "bing_search", feature = "brave_search", feature = "serpapi", feature = "exa_search", feature = "you_search"))]
mod search_tool_names {
    use super::*;
    #[test]
    fn test_search_tools_have_unique_names() {
        let t1 = TavilySearchTool::new();
        let t2 = GoogleSearchTool::new();
        let t3 = BingSearchTool::new();
        let t4 = BraveSearchTool::new();
        let t5 = SerpApiTool::new();
        let t6 = ExaSearchTool::new();
        let t7 = YouSearchTool::new();
        let names = vec![
            t1.name().to_string(),
            t2.name().to_string(),
            t3.name().to_string(),
            t4.name().to_string(),
            t5.name().to_string(),
            t6.name().to_string(),
            t7.name().to_string(),
        ];
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                assert_ne!(names[i], names[j], "search tool names must be unique");
            }
        }
    }
}
