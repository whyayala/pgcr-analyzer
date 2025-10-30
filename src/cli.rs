use crate::error::{BungieError, Result};
use crate::models::{ActivityType, FireteamAnalysis};
use crate::api_client::BungieApiClient;
use crate::analyzer::CarnageReportAnalyzer;
use clap::{Parser, Subcommand};
use std::str::FromStr;

/// Command-line interface for the Bungie Post-Game Carnage Report Analyzer
/// Provides a user-friendly way to interact with the Bungie API and analyze gameplay data
#[derive(Parser)]
#[command(name = "bungie-analyzer")]
#[command(about = "A Rust application for analyzing Destiny 2 post-game carnage reports via Bungie API")]
#[command(version)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands for the CLI application
#[derive(Subcommand)]
pub enum Commands {
    /// Search for users by their Bungie display name
    Search {
        /// The display name to search for
        #[arg(short, long)]
        name: String,
    },
    /// Analyze post-game carnage reports for a specific user
    Analyze {
        /// The Bungie membership ID of the user to analyze
        #[arg(short, long)]
        user_id: String,
        /// The membership type (1=Xbox, 2=PlayStation, 3=Steam, 5=Stadia, 10=Battle.net, 254=Bungie)
        #[arg(short, long, default_value = "3")]
        membership_type: i32,
        /// Activity type to filter by (story, strike, crucible, gambit, raid, dungeon, other)
        #[arg(short, long)]
        activity_type: Option<String>,
        /// Number of recent activities to analyze (max 250)
        #[arg(short, long, default_value = "25")]
        count: u32,
        /// Output format (json, table, detailed)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Get detailed information about a specific activity
    Activity {
        /// The activity instance ID to analyze
        #[arg(short, long)]
        activity_id: String,
        /// The Bungie membership ID of the user to focus on
        #[arg(short, long)]
        user_id: String,
        /// Output format (json, table, detailed)
        #[arg(short, long, default_value = "detailed")]
        format: String,
    },
}

/// Main CLI handler that processes commands and coordinates API interactions
pub struct CliHandler {
    /// The Bungie API client for making requests
    api_client: BungieApiClient,
    /// The carnage report analyzer for processing data
    analyzer: CarnageReportAnalyzer,
}

impl CliHandler {
    /// Creates a new CLI handler with the configured API client
    /// 
    /// # Arguments
    /// * `api_client` - The configured Bungie API client
    /// 
    /// # Returns
    /// * `Self` - A new CLI handler instance
    pub fn new(api_client: BungieApiClient) -> Self {
        Self {
            api_client,
            analyzer: CarnageReportAnalyzer::new(),
        }
    }

    /// Executes the specified command and handles the response
    /// 
    /// # Arguments
    /// * `command` - The command to execute
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    pub async fn execute(&mut self, command: Commands) -> Result<()> {
        match command {
            Commands::Search { name } => {
                self.handle_search_command(name).await
            }
            Commands::Analyze { user_id, membership_type, activity_type, count, format } => {
                self.handle_analyze_command(user_id, membership_type, activity_type, count, format).await
            }
            Commands::Activity { activity_id, user_id, format } => {
                self.handle_activity_command(activity_id, user_id, format).await
            }
        }
    }

    /// Handles the user search command
    /// 
    /// # Arguments
    /// * `name` - The display name to search for
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn handle_search_command(&self, name: String) -> Result<()> {
        println!("Searching for users with display name: {}", name);
        
        match self.api_client.search_users(&name).await {
            Ok(search_response) => {
                if search_response.search_results.is_empty() {
                    println!("No users found with display name: {}", name);
                } else {
                    println!("Found {} user(s):", search_response.total_results);
                    println!("{:<20} {:<15} {:<10}", "Display Name", "Membership ID", "Type");
                    println!("{}", "-".repeat(50));
                    
                    for user in search_response.search_results {
                        let membership_type_name = self.get_membership_type_name(user.membership_type);
                        println!("{:<20} {:<15} {}", user.display_name, user.membership_id, membership_type_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to search users: {}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }

    /// Handles the analyze command for processing multiple carnage reports
    /// 
    /// # Arguments
    /// * `user_id` - The user's Bungie membership ID
    /// * `membership_type` - The user's membership type
    /// * `activity_type` - Optional activity type filter
    /// * `count` - Number of activities to analyze
    /// * `format` - Output format preference
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn handle_analyze_command(
        &mut self,
        user_id: String,
        membership_type: i32,
        activity_type: Option<String>,
        count: u32,
        format: String,
    ) -> Result<()> {
        println!("Analyzing post-game carnage reports for user: {}", user_id);
        
        // Parse activity type if provided
        let parsed_activity_type = if let Some(activity_type_str) = activity_type {
            match ActivityType::from_str(&activity_type_str.to_lowercase()) {
                Ok(at) => Some(at),
                Err(_) => {
                    eprintln!("Invalid activity type: {}. Valid types: story, strike, crucible, gambit, raid, dungeon, other", activity_type_str);
                    return Err(BungieError::InvalidActivityType {
                        activity_type: activity_type_str,
                    });
                }
            }
        } else {
            None
        };

        // In the analyze method, before calling get_post_game_carnage_reports:
        let character_ids = self.api_client.get_characters(membership_type, &user_id).await?;

        if character_ids.is_empty() {
            println!("No characters found for this user.");
            return Ok(());
        }

        // Use the first character ID (already a String)
        let character_id = &character_ids[0];

        // Get post-game carnage reports
        let reports = self.api_client
            .get_post_game_carnage_reports(membership_type, &user_id, &character_id, parsed_activity_type, Some(count))
            .await?;

        if reports.is_empty() {
            println!("No post-game carnage reports found for the specified criteria.");
            return Ok(());
        }

        println!("Found {} post-game carnage report(s)", reports.len());

        // Analyze each report
        for (index, report) in reports.iter().enumerate() {
            println!("\n--- Report {} ---", index + 1);
            
            match self.analyzer.analyze_report(report, &user_id).await {
                Ok(analysis) => {
                    self.display_analysis(&analysis, &format);
                }
                Err(e) => {
                    eprintln!("Failed to analyze report {}: {}", index + 1, e);
                }
            }
        }

        Ok(())
    }

    /// Handles the activity command for analyzing a specific activity
    /// 
    /// # Arguments
    /// * `activity_id` - The activity instance ID
    /// * `user_id` - The user to focus on in the analysis
    /// * `format` - Output format preference
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn handle_activity_command(
        &mut self,
        activity_id: String,
        user_id: String,
        format: String,
    ) -> Result<()> {
        println!("Analyzing activity: {}", activity_id);
        
        // Get detailed activity information
        let report = self.api_client.get_activity_details(&activity_id).await?;
        
        // Analyze the report
        match self.analyzer.analyze_report(&report, &user_id).await {
            Ok(analysis) => {
                self.display_analysis(&analysis, &format);
            }
            Err(e) => {
                eprintln!("Failed to analyze activity: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Displays the analysis results in the specified format
    /// 
    /// # Arguments
    /// * `analysis` - The fireteam analysis to display
    /// * `format` - The display format (json, table, detailed)
    fn display_analysis(&self, analysis: &FireteamAnalysis, format: &str) {
        match format.to_lowercase().as_str() {
            "json" => {
                if let Ok(json) = serde_json::to_string_pretty(analysis) {
                    println!("{}", json);
                } else {
                    eprintln!("Failed to serialize analysis to JSON");
                }
            }
            "table" => {
                self.display_table_format(analysis);
            }
            "detailed" => {
                self.display_detailed_format(analysis);
            }
            _ => {
                eprintln!("Invalid format: {}. Using detailed format.", format);
                self.display_detailed_format(analysis);
            }
        }
    }

    /// Displays analysis in table format
    /// 
    /// # Arguments
    /// * `analysis` - The analysis to display
    fn display_table_format(&self, analysis: &FireteamAnalysis) {
        println!("\n=== FIRETEAM ANALYSIS ===");
        println!("Activity Type: {:?}", analysis.activity_summary.activity_type);
        println!("Completion Status: {}", analysis.activity_summary.completion_status);
        println!("Total Fireteam Kills: {}", analysis.activity_summary.total_fireteam_kills);
        println!("Total Fireteam Deaths: {}", analysis.activity_summary.total_fireteam_deaths);
        
        println!("\n--- Primary User ---");
        self.display_user_summary(&analysis.primary_user);
        
        if !analysis.fireteam_members.is_empty() {
            println!("\n--- Fireteam Members ---");
            for member in &analysis.fireteam_members {
                self.display_user_summary(member);
            }
        }
    }

    /// Displays analysis in detailed format
    /// 
    /// # Arguments
    /// * `analysis` - The analysis to display
    fn display_detailed_format(&self, analysis: &FireteamAnalysis) {
        println!("\n=== DETAILED FIRETEAM ANALYSIS ===");
        
        // Activity summary
        println!("\nACTIVITY SUMMARY:");
        println!("  Type: {:?}", analysis.activity_summary.activity_type);
        println!("  Duration: {}", analysis.activity_summary.duration);
        println!("  Status: {}", analysis.activity_summary.completion_status);
        println!("  Total Kills: {}", analysis.activity_summary.total_fireteam_kills);
        println!("  Total Deaths: {}", analysis.activity_summary.total_fireteam_deaths);
        
        // Primary user details
        println!("\nPRIMARY USER:");
        self.display_user_detailed(&analysis.primary_user);
        
        // Fireteam members
        if !analysis.fireteam_members.is_empty() {
            println!("\nFIRETEAM MEMBERS:");
            for (index, member) in analysis.fireteam_members.iter().enumerate() {
                println!("\n  Member {}:", index + 1);
                self.display_user_detailed(member);
            }
        }
    }

    /// Displays a summary of user information in table format
    /// 
    /// # Arguments
    /// * `user` - The user analysis to display
    fn display_user_summary(&self, user: &crate::models::UserAnalysis) {
        println!("  {} ({}): K/D {:.2}, Score {:.0}, Performance {:.2}",
                user.user_info.display_name,
                user.character_info.character_class,
                if user.kill_attribution.weapon_kills + user.kill_attribution.ability_kills > 0 {
                    (user.kill_attribution.weapon_kills + user.kill_attribution.ability_kills) as f64 / 
                    (user.kill_attribution.total_kills - user.kill_attribution.weapon_kills - user.kill_attribution.ability_kills).max(1) as f64
                } else { 0.0 },
                0.0, // Score would need to be extracted from participant data
                user.performance_score);
    }

    /// Displays detailed user information
    /// 
    /// # Arguments
    /// * `user` - The user analysis to display
    fn display_user_detailed(&self, user: &crate::models::UserAnalysis) {
        println!("    Name: {}", user.user_info.display_name);
        println!("    Class: {}", user.character_info.character_class);
        println!("    Level: {}", user.character_info.character_level);
        println!("    Light Level: {}", user.character_info.light_level);
        println!("    Performance Score: {:.2}", user.performance_score);
        
        println!("    KILL ATTRIBUTION:");
        println!("      Total Kills: {}", user.kill_attribution.total_kills);
        println!("      Weapon Kills: {}", user.kill_attribution.weapon_kills);
        println!("      Ability Kills: {}", user.kill_attribution.ability_kills);
        println!("      Environmental Kills: {}", user.kill_attribution.environmental_kills);
        
        if !user.kill_attribution.weapon_breakdown.is_empty() {
            println!("      Weapon Breakdown:");
            for (weapon, kills) in &user.kill_attribution.weapon_breakdown {
                println!("        {}: {}", weapon, kills);
            }
        }
        
        if !user.kill_attribution.ability_breakdown.is_empty() {
            println!("      Ability Breakdown:");
            for (ability, kills) in &user.kill_attribution.ability_breakdown {
                println!("        {}: {}", ability, kills);
            }
        }
        
        println!("    EQUIPMENT:");
        if let Some(primary) = &user.equipment.primary_weapon {
            println!("      Primary: {} ({})", primary.name, primary.weapon_type);
        }
        if let Some(secondary) = &user.equipment.secondary_weapon {
            println!("      Secondary: {} ({})", secondary.name, secondary.weapon_type);
        }
        if let Some(heavy) = &user.equipment.heavy_weapon {
            println!("      Heavy: {} ({})", heavy.name, heavy.weapon_type);
        }
    }

    /// Gets a human-readable name for membership types
    /// 
    /// # Arguments
    /// * `membership_type` - The numeric membership type
    /// 
    /// # Returns
    /// * `String` - Human-readable membership type name
    fn get_membership_type_name(&self, membership_type: i32) -> String {
        match membership_type {
            1 => "Xbox".to_string(),
            2 => "PlayStation".to_string(),
            3 => "Steam".to_string(),
            5 => "Stadia".to_string(),
            10 => "Battle.net".to_string(),
            254 => "Bungie".to_string(),
            _ => format!("Unknown ({})", membership_type),
        }
    }
}

/// Implements FromStr for ActivityType to enable parsing from command line arguments
impl FromStr for ActivityType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "story" => Ok(ActivityType::Story),
            "strike" => Ok(ActivityType::Strike),
            "crucible" => Ok(ActivityType::Crucible),
            "gambit" => Ok(ActivityType::Gambit),
            "raid" => Ok(ActivityType::Raid),
            "dungeon" => Ok(ActivityType::Dungeon),
            "public_event" | "public" => Ok(ActivityType::PublicEvent),
            "other" => Ok(ActivityType::Other),
            _ => Err(format!("Invalid activity type: {}", s)),
        }
    }
}
