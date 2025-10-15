/// Bungie Post-Game Carnage Report Analyzer
/// 
/// A comprehensive Rust application for analyzing Destiny 2 post-game carnage reports
/// through the Bungie API. This library provides functionality to:
/// 
/// - Search for users by display name
/// - Retrieve and analyze post-game carnage reports
/// - Extract equipment and loadout information
/// - Analyze kill attribution (weapon vs ability kills)
/// - Generate comprehensive fireteam performance reports
/// 
/// # Features
/// 
/// - **User Search**: Find players by their Bungie display name
/// - **Activity Analysis**: Analyze recent activities with filtering by activity type
/// - **Equipment Tracking**: Extract detailed equipment information for users and fireteam
/// - **Kill Attribution**: Distinguish between weapon kills, ability kills, and environmental kills
/// - **Fireteam Analysis**: Comprehensive analysis of all fireteam members' performance
/// - **Multiple Output Formats**: JSON, table, and detailed text output formats
/// - **Rate Limiting**: Built-in rate limiting to respect Bungie API limits
/// - **Error Handling**: Comprehensive error handling with helpful error messages
/// 
/// # Usage
/// 
/// ## Command Line Interface
/// 
/// The application provides a command-line interface with the following commands:
/// 
/// ```bash
/// # Search for users by display name
/// bungie-analyzer search --name "PlayerName"
/// 
/// # Analyze recent activities for a user
/// bungie-analyzer analyze --user-id "123456789" --membership-type 4 --activity-type crucible --count 10
/// 
/// # Analyze a specific activity
/// bungie-analyzer activity --activity-id "1234567890" --user-id "123456789"
/// ```
/// 
/// ## Library Usage
/// 
/// ```rust
/// use bungie_pgcr_analyzer::api_client::create_client_from_env;
/// use bungie_pgcr_analyzer::analyzer::CarnageReportAnalyzer;
/// use bungie_pgcr_analyzer::models::ActivityType;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create API client
///     let client = create_client_from_env()?;
///     
///     // Search for a user
///     let search_results = client.search_users("PlayerName").await?;
///     
///     // Get their recent activities
///     let reports = client.get_post_game_carnage_reports(
///         4, // Steam membership type
///         &search_results.search_results[0].membership_id,
///         Some(ActivityType::Crucible),
///         Some(10)
///     ).await?;
///     
///     // Analyze the reports
///     let mut analyzer = CarnageReportAnalyzer::new();
///     for report in reports {
///         let analysis = analyzer.analyze_report(&report, &user_id).await?;
///         println!("Analysis: {:?}", analysis);
///     }
///     
///     Ok(())
/// }
/// ```
/// 
/// # Configuration
/// 
/// The application requires a Bungie API key to function. Set the `BUNGIE_API_KEY`
/// environment variable with your API key from the Bungie Developer Portal.
/// 
/// # API Rate Limiting
/// 
/// The application includes built-in rate limiting to respect Bungie's API limits.
/// By default, it includes a 100ms delay between requests to prevent hitting
/// rate limits. This can be configured in the `BungieApiConfig` structure.
/// 
/// # Error Handling
/// 
/// The application uses a comprehensive error handling system with custom error types
/// for different failure scenarios. All errors include helpful messages and context
/// to assist with debugging and user guidance.
pub mod api_client;
pub mod analyzer;
pub mod cli;
pub mod error;
pub mod models;

// Re-export commonly used types for convenience
pub use error::{BungieError, Result};
pub use models::{
    ActivityType, FireteamAnalysis, PostGameCarnageReport, UserInfo, UserSearchResponse
};
pub use api_client::{BungieApiClient, BungieApiConfig};
pub use analyzer::CarnageReportAnalyzer;
