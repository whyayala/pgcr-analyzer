use bungie_pgcr_analyzer::api_client::create_client_from_env;
use bungie_pgcr_analyzer::cli::{Cli, CliHandler};
use bungie_pgcr_analyzer::error::Result;
use clap::Parser;
use tracing::{info, error};

/// Main entry point for the Bungie Post-Game Carnage Report Analyzer
/// 
/// This application provides a command-line interface for analyzing Destiny 2
/// post-game carnage reports through the Bungie API. It allows users to:
/// - Search for players by display name
/// - Analyze recent activities for specific players
/// - Examine detailed equipment and kill attribution data
/// - View comprehensive fireteam performance analysis
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging for better debugging and user feedback
    init_logging()?;
    
    info!("Starting Bungie Post-Game Carnage Report Analyzer");
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Create API client from environment variables
    let api_client = match create_client_from_env() {
        Ok(client) => {
            info!("Successfully created Bungie API client");
            client
        }
        Err(e) => {
            error!("Failed to create API client: {}", e);
            eprintln!("\nError: {}", e);
            eprintln!("\nPlease ensure you have set the BUNGIE_API_KEY environment variable.");
            eprintln!("You can obtain an API key from: https://www.bungie.net/en/Application");
            eprintln!("\nExample usage:");
            eprintln!("  export BUNGIE_API_KEY=your_api_key_here");
            eprintln!("  bungie-analyzer search --name \"YourDisplayName\"");
            return Err(e);
        }
    };
    
    // Create CLI handler and execute the requested command
    let mut cli_handler = CliHandler::new(api_client);
    
    match cli_handler.execute(cli.command).await {
        Ok(()) => {
            info!("Command executed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Command failed: {}", e);
            eprintln!("\nError: {}", e);
            
            // Provide helpful error messages for common issues
            match e {
                bungie_pgcr_analyzer::error::BungieError::ApiError { message, error_code } => {
                    eprintln!("\nBungie API Error Details:");
                    eprintln!("  Message: {}", message);
                    eprintln!("  Error Code: {}", error_code);
                    
                    match error_code {
                        1 => eprintln!("  This usually indicates an invalid API key or insufficient permissions."),
                        5 => eprintln!("  This usually indicates the requested resource was not found."),
                        1627 => eprintln!("  This usually indicates the user was not found."),
                        _ => eprintln!("  Check the Bungie API documentation for more details about this error."),
                    }
                }
                bungie_pgcr_analyzer::error::BungieError::RateLimitExceeded { retry_after } => {
                    eprintln!("\nRate limit exceeded. Please wait {} seconds before trying again.", retry_after);
                }
                bungie_pgcr_analyzer::error::BungieError::UserNotFound { bungie_id } => {
                    eprintln!("\nUser not found: {}", bungie_id);
                    eprintln!("  Make sure you're using the correct Bungie membership ID.");
                    eprintln!("  You can find this by using the search command first.");
                }
                _ => {
                    eprintln!("\nFor help with this error, check the application documentation or contact support.");
                }
            }
            
            std::process::exit(1);
        }
    }
}

/// Initializes the logging system for the application
/// 
/// Sets up structured logging with appropriate levels and formatting
/// to help with debugging and monitoring application behavior
/// 
/// # Returns
/// * `Result<()>` - Success or error result
fn init_logging() -> Result<()> {
    // Initialize tracing subscriber with environment-based configuration
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();
    
    Ok(())
}
