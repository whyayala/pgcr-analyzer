use crate::error::{BungieError, Result};
use crate::models::{
    ActivityType, PostGameCarnageReport, UserSearchResponse,
};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// Configuration for the Bungie API client
/// Contains API endpoints, authentication details, and request settings
#[derive(Debug, Clone)]
pub struct BungieApiConfig {
    /// Base URL for the Bungie API
    pub base_url: String,
    /// API key for authentication (required for all requests)
    pub api_key: String,
    /// Request timeout duration
    pub timeout: Duration,
    /// Rate limiting delay between requests (in milliseconds)
    pub rate_limit_delay: u64,
}

impl Default for BungieApiConfig {
    /// Provides default configuration for the Bungie API client
    /// Uses production API endpoints and reasonable timeouts
    fn default() -> Self {
        Self {
            base_url: "https://www.bungie.net/Platform".to_string(),
            api_key: String::new(),
            timeout: Duration::from_secs(30),
            rate_limit_delay: 100, // 100ms delay between requests
        }
    }
}

/// Main client for interacting with the Bungie API
/// Handles authentication, rate limiting, and all API requests
#[derive(Debug, Clone)]
pub struct BungieApiClient {
    /// HTTP client for making requests
    client: Client,
    /// Configuration for the API client
    config: BungieApiConfig,
}

impl BungieApiClient {
    /// Creates a new Bungie API client with the provided configuration
    /// 
    /// # Arguments
    /// * `config` - Configuration containing API key and other settings
    /// 
    /// # Returns
    /// * `Result<Self>` - The configured API client or an error
    pub fn new(config: BungieApiConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(BungieError::ConfigError {
                message: "API key is required".to_string(),
            });
        }

        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent("bungie-pgcr-analyzer/0.1.0")
            .build()
            .map_err(|e| BungieError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self { client, config })
    }

    /// Searches for users by their Bungie display name
    /// 
    /// # Arguments
    /// * `display_name` - The display name to search for
    /// 
    /// # Returns
    /// * `Result<UserSearchResponse>` - Search results or an error
    pub async fn search_users(&self, display_name: &str) -> Result<UserSearchResponse> {
        let url = format!("{}/Destiny2/SearchDestinyPlayer/-1/{}/", 
                         self.config.base_url, 
                         urlencoding::encode(display_name));

        let response = self.make_request(&url).await?;
        self.parse_user_search_response(response).await
    }

    /// Retrieves post-game carnage reports for a specific user
    /// 
    /// # Arguments
    /// * `membership_type` - The membership type (1=Xbox, 2=PlayStation, 4=Steam, etc.)
    /// * `membership_id` - The user's membership ID
    /// * `activity_type` - Optional filter for activity type
    /// * `count` - Number of reports to retrieve (max 250)
    /// 
    /// # Returns
    /// * `Result<Vec<PostGameCarnageReport>>` - List of carnage reports or an error
    pub async fn get_post_game_carnage_reports(
        &self,
        membership_type: i32,
        membership_id: &str,
        character_id: &str, // Add this parameter
        activity_type: Option<ActivityType>,
        count: Option<u32>,
    ) -> Result<Vec<PostGameCarnageReport>> {
        let count = count.unwrap_or(25).min(250);
        let url = format!(
            "{}/Destiny2/{}/Account/{}/Character/{}/Stats/Activities/?count={}",
            self.config.base_url, membership_type, membership_id, character_id, count
        );
    
        let response = self.make_request(&url).await?;
        let mut reports = self.parse_activities_response(response).await?;
    
        // Filter by activity type if specified
        if let Some(activity_type) = activity_type {
            reports = self.filter_reports_by_activity_type(reports, activity_type).await?;
        }
    
        Ok(reports)
    }

    /// Retrieves detailed post-game carnage report for a specific activity
    /// 
    /// # Arguments
    /// * `activity_id` - The unique activity instance ID
    /// 
    /// # Returns
    /// * `Result<PostGameCarnageReport>` - Detailed carnage report or an error
    pub async fn get_activity_details(&self, activity_id: &str) -> Result<PostGameCarnageReport> {
        let url = format!(
            "{}/Destiny2/Stats/PostGameCarnageReport/{}/",
            self.config.base_url, activity_id
        );

        let response = self.make_request(&url).await?;
        self.parse_carnage_report_response(response).await
    }

    /// Makes an authenticated HTTP request to the Bungie API
    /// Handles rate limiting, error responses, and response parsing
    /// 
    /// # Arguments
    /// * `url` - The API endpoint URL
    /// 
    /// # Returns
    /// * `Result<Value>` - Parsed JSON response or an error
    async fn make_request(&self, url: &str) -> Result<Value> {
        // Apply rate limiting delay
        if self.config.rate_limit_delay > 0 {
            std::thread::sleep(Duration::from_millis(self.config.rate_limit_delay));
        }
    
        let request = self
            .client
            .get(url)
            .header("X-API-Key", &self.config.api_key);
    
        let response = request.send().await?;
    
        // Check for rate limiting
        if let Some(retry_after) = response.headers().get("X-RateLimit-Remaining") {
            if let Ok(remaining) = retry_after.to_str() {
                if remaining == "0" {
                    if let Some(reset_time) = response.headers().get("X-RateLimit-Reset") {
                        if let Ok(reset_str) = reset_time.to_str() {
                            if let Ok(reset_secs) = reset_str.parse::<u64>() {
                                return Err(BungieError::RateLimitExceeded {
                                    retry_after: reset_secs,
                                });
                            }
                        }
                    }
                }
            }
        }
    
        let status = response.status();
        let response_text = response.text().await?;
    
        // Parse JSON response
        let json_response: Value = serde_json::from_str(&response_text)?;
        
        // Check for Bungie API errors
        if !status.is_success() {
            let error_message = json_response
                .get("Message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown API error")
                .to_string();
    
            let error_code = json_response
                .get("ErrorCode")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
    
            return Err(BungieError::ApiError {
                message: error_message,
                error_code,
            });
        }
    
        Ok(json_response)
    }

    /// Parses the user search response from the Bungie API
    /// 
    /// # Arguments
    /// * `response` - Raw JSON response from the API
    /// 
    /// # Returns
    /// * `Result<UserSearchResponse>` - Parsed user search results or an error
    async fn parse_user_search_response(&self, response: serde_json::Value) -> Result<UserSearchResponse> {
    
        let response_array = response
            .get("Response")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BungieError::ValidationError {
                field: "Response".to_string(),
                message: "Expected Response to be an array in user search response".to_string(),
            })?;
    
        let mut users = Vec::new();
        for item in response_array {
            let membership_id = item
                .get("membershipId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
    
            let display_name = item
                .get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
    
            let membership_type = item
                .get("membershipType")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
    
            // Only include entries with required fields
            if !membership_id.is_empty() && !display_name.is_empty() && membership_type != 0 {
                users.push(crate::models::UserInfo {
                    membership_id,
                    display_name,
                    membership_type,
                });
            }
        }
    
        Ok(UserSearchResponse {
            total_results: users.len() as i32,
            search_results: users,
            has_more: false,
        })
    }

    /// Parses the activities response to extract activity IDs
    /// 
    /// # Arguments
    /// * `response` - Raw JSON response from the activities API
    /// 
    /// # Returns
    /// * `Result<Vec<PostGameCarnageReport>>` - List of carnage reports or an error
    async fn parse_activities_response(&self, response: Value) -> Result<Vec<PostGameCarnageReport>> {
        
        let response_data = response
            .get("Response")
            .ok_or_else(|| BungieError::ValidationError {
                field: "Response".to_string(),
                message: "Missing Response field in activities response".to_string(),
            })?;
    
        let activities = response_data
            .get("activities")
            .ok_or_else(|| BungieError::ValidationError {
                field: "activities".to_string(),
                message: "Missing activities field in activities response".to_string(),
            })?;

        let mut reports = Vec::new();
        if let Some(activities_array) = activities.as_array() {
            for activity in activities_array {
                if let Some(instance_id) = activity.get("activityDetails").and_then(|v| v.get("instanceId").and_then(|v| v.as_str())) {
                    match self.get_activity_details(instance_id).await {
                        Ok(report) => reports.push(report),
                        Err(e) => {
                            // Log error but continue processing other activities
                            eprintln!("Failed to get details for activity {}: {}", instance_id, e);
                        }
                    }
                }
            }
        }

        Ok(reports)
    }

    /// Parses a post-game carnage report response
    /// 
    /// # Arguments
    /// * `response` - Raw JSON response from the carnage report API
    /// 
    /// # Returns
    /// * `Result<PostGameCarnageReport>` - Parsed carnage report or an error
    async fn parse_carnage_report_response(&self, response: Value) -> Result<PostGameCarnageReport> {        
        let response_data = response
            .get("Response")
            .ok_or_else(|| BungieError::ValidationError {
                field: "Response".to_string(),
                message: "Missing Response field in carnage report response".to_string(),
            })?;
   
        serde_json::from_value(response_data.clone())
            .map_err(|e| BungieError::ValidationError {
                field: "PostGameCarnageReport".to_string(),
                message: format!("Failed to parse carnage report: {}", e),
            })
    }

    /// Filters post-game carnage reports by activity type
    /// 
    /// # Arguments
    /// * `reports` - List of carnage reports to filter
    /// * `activity_type` - The activity type to filter by
    /// 
    /// # Returns
    /// * `Result<Vec<PostGameCarnageReport>>` - Filtered reports or an error
    async fn filter_reports_by_activity_type(
        &self,
        reports: Vec<PostGameCarnageReport>,
        activity_type: ActivityType,
    ) -> Result<Vec<PostGameCarnageReport>> {
        // This is a simplified filter - in a real implementation, you would
        // need to map activity mode values to ActivityType enum values
        // based on Bungie's activity definitions
        let filtered_reports: Vec<PostGameCarnageReport> = reports
            .into_iter()
            .filter(|report| {
                // Map Bungie activity modes to our ActivityType enum
                match report.activity_details.mode {
                    2 => activity_type == ActivityType::Story,
                    3 => activity_type == ActivityType::Strike,
                    4 => activity_type == ActivityType::Raid,
                    5 => activity_type == ActivityType::Crucible,
                    6 => activity_type == ActivityType::Gambit,
                    7 => activity_type == ActivityType::Dungeon,
                    _ => activity_type == ActivityType::Other,
                }
            })
            .collect();

        Ok(filtered_reports)
    }

    /// Gets the user's characters
    pub async fn get_characters(&self, membership_type: i32, membership_id: &str) -> Result<Vec<String>> {
        let url = format!(
            "{}/Destiny2/{}/Profile/{}/?components=200",
            self.config.base_url, membership_type, membership_id
        );

        let response = self.make_request(&url).await?;
        
        // Parse character IDs from the response
        if let Some(characters) = response.get("Response")
            .and_then(|r| r.get("characters"))
            .and_then(|c| c.get("data"))
            .and_then(|d| d.as_object()) {
            
            let character_ids: Vec<String> = characters.keys()
                .map(|k| k.to_string())
                .collect();
            
            Ok(character_ids)
        } else {
            Ok(vec![])
        }
    }
}

/// Helper function to create a Bungie API client with environment variables
/// Reads API key from BUNGIE_API_KEY environment variable
/// 
/// # Returns
/// * `Result<BungieApiClient>` - Configured client or an error
pub fn create_client_from_env() -> Result<BungieApiClient> {
    let api_key = std::env::var("BUNGIE_API_KEY")
        .map_err(|_| BungieError::ConfigError {
            message: "BUNGIE_API_KEY environment variable not set".to_string(),
        })?;

    let config = BungieApiConfig {
        api_key,
        ..Default::default()
    };

    BungieApiClient::new(config)
}

