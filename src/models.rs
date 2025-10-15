use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Bungie user's basic information
/// This structure contains the essential user data needed to identify and query users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(rename = "membershipId")]
    pub membership_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "membershipType")]
    pub membership_type: i32,
}

/// Represents the response from Bungie's user search API
/// Contains the search results and metadata about the search operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResponse {
    /// The search results containing user information
    pub search_results: Vec<UserInfo>,
    /// Total number of results found
    pub total_results: i32,
    /// Whether there are more results available
    pub has_more: bool,
}

/// Represents different types of activities in Destiny 2
/// Used to filter post-game carnage reports by activity type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActivityType {
    /// Story missions and campaigns
    Story,
    /// Strikes and nightfalls
    Strike,
    /// Crucible PvP matches
    Crucible,
    /// Gambit matches
    Gambit,
    /// Raid encounters
    Raid,
    /// Dungeon encounters
    Dungeon,
    /// Public events and patrols
    PublicEvent,
    /// All other activity types
    Other,
}

/// Represents a post-game carnage report from the Bungie API
/// Contains comprehensive information about a completed activity including player performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostGameCarnageReport {
    /// Unique identifier for this activity instance
    #[serde(rename = "activityDetails")]
    pub activity_details: ActivityDetails,
    /// Timestamp when the activity was completed
    pub period: String,
    /// List of all participants in the activity
    pub entries: Vec<ParticipantEntry>,
    /// Teams involved in the activity (for PvP activities)
    pub teams: Option<Vec<Team>>,
}

/// Details about the specific activity instance
/// Contains metadata about the activity type, location, and modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetails {
    #[serde(rename = "referenceId")]
    pub reference_id: u32,
    #[serde(rename = "directorActivityHash")]
    pub director_activity_hash: u32,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    pub mode: i32,
    #[serde(default)]
    pub modes: Option<Vec<i32>>,
}

/// Represents a participant in the post-game carnage report
/// Contains detailed performance data for each player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantEntry {
    pub standing: i32,
    pub score: Score,
    pub player: PlayerInfo,
    #[serde(rename = "characterId")]
    pub character_id: String,
    pub extended: ExtendedData,
    pub values: HashMap<String, StatValue>,  // Main stats are here!
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedData {
    pub values: HashMap<String, StatValue>,  // Extended stats like weaponKillsGrenade
    pub weapons: Vec<WeaponStats>,
    #[serde(default, rename = "scoreboardValues")]
    pub scoreboard_values: Option<HashMap<String, StatValue>>,
}

/// Player membership information
/// Links the participant to their Bungie account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    #[serde(rename = "destinyUserInfo")]
    pub destiny_user_info: UserInfo,
    #[serde(rename = "characterClass")]
    pub character_class: String,
    #[serde(rename = "characterLevel")]
    pub character_level: i32,
    #[serde(rename = "lightLevel")]
    pub light_level: i32,
}

/// Score information for the participant
/// Contains various scoring metrics depending on activity type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub basic: ScoreValue,
    #[serde(default)]
    pub precision: Option<ScoreValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatValue {
    pub basic: ScoreValue,
}

/// Individual score value with display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreValue {
    pub value: f64,
    #[serde(rename = "displayValue")]
    pub display_value: String,
}

/// Extended statistics for the participant
/// Contains detailed performance metrics including kills, deaths, assists, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedValues {
    pub values: std::collections::HashMap<String, StatValue>,
    pub weapons: Vec<WeaponStats>,
}

/// Statistics for a specific weapon used during the activity
/// Contains kill counts and usage statistics for each weapon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponStats {
    #[serde(rename = "referenceId")]
    pub reference_id: u32,
    pub values: HashMap<String, StatValue>,
}

/// Team information for activities with teams (primarily PvP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Team ID
    pub team_id: i32,
    /// Standing of the team
    pub standing: i32,
    /// Score of the team
    pub score: Score,
}

/// Detailed equipment information for a player
/// Represents what gear a player had equipped during the activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    /// Primary weapon equipped
    pub primary_weapon: Option<Weapon>,
    /// Secondary weapon equipped
    pub secondary_weapon: Option<Weapon>,
    /// Heavy weapon equipped
    pub heavy_weapon: Option<Weapon>,
    /// Helmet equipped
    pub helmet: Option<Armor>,
    /// Gauntlets equipped
    pub gauntlets: Option<Armor>,
    /// Chest armor equipped
    pub chest_armor: Option<Armor>,
    /// Leg armor equipped
    pub leg_armor: Option<Armor>,
    /// Class item equipped
    pub class_item: Option<Armor>,
    /// Ghost shell equipped
    pub ghost_shell: Option<Ghost>,
    /// Vehicle equipped (sparrow)
    pub vehicle: Option<Vehicle>,
}

/// Weapon information with detailed stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    /// Item hash for the weapon
    pub item_hash: u32,
    /// Name of the weapon
    pub name: String,
    /// Weapon type (Auto Rifle, Hand Cannon, etc.)
    pub weapon_type: String,
    /// Damage type (Kinetic, Solar, Arc, Void, Stasis, Strand)
    pub damage_type: String,
    /// Power level of the weapon
    pub power_level: i32,
}

/// Armor piece information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Armor {
    /// Item hash for the armor piece
    pub item_hash: u32,
    /// Name of the armor piece
    pub name: String,
    /// Armor slot (Helmet, Gauntlets, etc.)
    pub slot: String,
    /// Power level of the armor
    pub power_level: i32,
    /// Total stats of the armor piece
    pub stats: HashMap<String, i32>,
}

/// Ghost shell information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ghost {
    /// Item hash for the ghost shell
    pub item_hash: u32,
    /// Name of the ghost shell
    pub name: String,
    /// Power level of the ghost
    pub power_level: i32,
}

/// Vehicle (sparrow) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    /// Item hash for the vehicle
    pub item_hash: u32,
    /// Name of the vehicle
    pub name: String,
    /// Power level of the vehicle
    pub power_level: i32,
}

/// Kill attribution data showing what caused each kill
/// Distinguishes between weapon kills and ability kills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillAttribution {
    /// Total number of kills by this player
    pub total_kills: i32,
    /// Kills attributed to weapons
    pub weapon_kills: i32,
    /// Kills attributed to abilities (grenades, melee, supers, etc.)
    pub ability_kills: i32,
    /// Kills attributed to environmental damage
    pub environmental_kills: i32,
    /// Detailed breakdown by weapon type
    pub weapon_breakdown: HashMap<String, i32>,
    /// Detailed breakdown by ability type
    pub ability_breakdown: HashMap<String, i32>,
}

/// Fireteam analysis results
/// Contains comprehensive analysis of all fireteam members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireteamAnalysis {
    /// The primary user being analyzed
    pub primary_user: UserAnalysis,
    /// Analysis of all fireteam members
    pub fireteam_members: Vec<UserAnalysis>,
    /// Activity summary statistics
    pub activity_summary: ActivitySummary,
}

/// Individual user analysis within the fireteam
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnalysis {
    /// User information
    pub user_info: UserInfo,
    /// Character information
    pub character_info: PlayerInfo,
    /// Equipment analysis
    pub equipment: Equipment,
    /// Kill attribution analysis
    pub kill_attribution: KillAttribution,
    /// Overall performance score
    pub performance_score: f64,
}

/// Summary statistics for the entire activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    /// Type of activity performed
    pub activity_type: ActivityType,
    /// Total duration of the activity
    pub duration: String,
    /// Total kills by the fireteam
    pub total_fireteam_kills: i32,
    /// Total deaths by the fireteam
    pub total_fireteam_deaths: i32,
    /// Success/failure status of the activity
    pub completion_status: String,
}
