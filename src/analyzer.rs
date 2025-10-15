use crate::error::{BungieError, Result};
use crate::models::{
    ActivitySummary, ActivityType, Equipment, FireteamAnalysis, KillAttribution, 
    PostGameCarnageReport, UserAnalysis, Weapon, WeaponStats,
};
use std::collections::HashMap;

/// Main analyzer for processing post-game carnage reports
/// Extracts equipment information, kill attribution, and fireteam analysis
pub struct CarnageReportAnalyzer {
    /// Cache for weapon/armor definitions to avoid repeated API calls
    item_definitions: HashMap<u32, String>,
}

impl CarnageReportAnalyzer {
    /// Creates a new carnage report analyzer
    /// 
    /// # Returns
    /// * `Self` - A new analyzer instance
    pub fn new() -> Self {
        Self {
            item_definitions: HashMap::new(),
        }
    }

    /// Analyzes a post-game carnage report and extracts comprehensive fireteam data
    /// 
    /// # Arguments
    /// * `report` - The post-game carnage report to analyze
    /// * `target_user_id` - The Bungie ID of the user we're primarily interested in
    /// 
    /// # Returns
    /// * `Result<FireteamAnalysis>` - Complete analysis of the fireteam or an error
    pub async fn analyze_report(
        &mut self,
        report: &PostGameCarnageReport,
        target_user_id: &str,
    ) -> Result<FireteamAnalysis> {
        // Find the target user in the participant list
        let target_user = report
            .entries
            .iter()
            .find(|entry| entry.player.destiny_user_info.membership_id == target_user_id)
            .ok_or_else(|| BungieError::UserNotFound {
                bungie_id: target_user_id.to_string(),
            })?;

        // Analyze the target user
        let primary_user_analysis = self.analyze_user(report, target_user).await?;

        // Analyze all other fireteam members
        let mut fireteam_members = Vec::new();
        for entry in &report.entries {
            if entry.player.destiny_user_info.membership_id != target_user_id {
                match self.analyze_user(report, entry).await {
                    Ok(analysis) => fireteam_members.push(analysis),
                    Err(e) => {
                        eprintln!("Failed to analyze fireteam member {}: {}", 
                                 entry.player.destiny_user_info.display_name, e);
                    }
                }
            }
        }

        // Create activity summary
        let activity_summary = self.create_activity_summary(report).await?;

        Ok(FireteamAnalysis {
            primary_user: primary_user_analysis,
            fireteam_members,
            activity_summary,
        })
    }

    /// Analyzes an individual user within a carnage report
    /// 
    /// # Arguments
    /// * `report` - The complete carnage report
    /// * `participant` - The participant entry to analyze
    /// 
    /// # Returns
    /// * `Result<UserAnalysis>` - Complete user analysis or an error
    async fn analyze_user(
        &mut self,
        _report: &PostGameCarnageReport,
        participant: &crate::models::ParticipantEntry,
    ) -> Result<UserAnalysis> {
        // Extract equipment information
        let equipment = self.extract_equipment(participant).await?;

        // Analyze kill attribution
        let kill_attribution = self.analyze_kill_attribution(participant).await?;

        // Calculate performance score
        let performance_score = self.calculate_performance_score(participant).await?;

        Ok(UserAnalysis {
            user_info: participant.player.destiny_user_info.clone(),
            character_info: participant.player.clone(),
            equipment,
            kill_attribution,
            performance_score,
        })
    }

    /// Extracts equipment information for a participant
    /// Note: This is a simplified implementation. In a full implementation,
    /// you would need to make additional API calls to get detailed equipment data
    /// 
    /// # Arguments
    /// * `participant` - The participant to extract equipment from
    /// 
    /// # Returns
    /// * `Result<Equipment>` - Equipment information or an error
    async fn extract_equipment(
        &mut self,
        participant: &crate::models::ParticipantEntry,
    ) -> Result<Equipment> {
        // In a real implementation, you would need to:
        // 1. Make API calls to get character inventory
        // 2. Parse equipment from the character data
        // 3. Resolve item hashes to actual item definitions
        
        // For now, we'll create a placeholder equipment structure
        // based on the weapon stats available in the carnage report
        let mut weapons = Vec::new();
        
        // Extract weapon information from the extended values
        for weapon_stats in &participant.extended.weapons {
            if let Some(weapon) = self.create_weapon_from_stats(weapon_stats).await? {
                weapons.push(weapon);
            }
        }

        // Create equipment structure with available weapons
        let equipment = Equipment {
            primary_weapon: weapons.first().cloned(),
            secondary_weapon: weapons.get(1).cloned(),
            heavy_weapon: weapons.get(2).cloned(),
            helmet: None, // Would require additional API calls
            gauntlets: None,
            chest_armor: None,
            leg_armor: None,
            class_item: None,
            ghost_shell: None,
            vehicle: None,
        };

        Ok(equipment)
    }

    /// Creates a weapon object from weapon statistics
    /// 
    /// # Arguments
    /// * `weapon_stats` - Statistics for the weapon
    /// 
    /// # Returns
    /// * `Result<Option<Weapon>>` - Weapon object or None if creation fails
    async fn create_weapon_from_stats(&mut self, weapon_stats: &WeaponStats) -> Result<Option<Weapon>> {
        // Get weapon name from cache or create placeholder
        let weapon_name = self.get_item_name(weapon_stats.reference_id).await;

        // Determine weapon type based on reference ID (simplified mapping)
        let weapon_type = self.determine_weapon_type(weapon_stats.reference_id).await;
        let damage_type = self.determine_damage_type(weapon_stats.reference_id).await;

        Ok(Some(Weapon {
            item_hash: weapon_stats.reference_id,
            name: weapon_name,
            weapon_type,
            damage_type,
            power_level: 0, // Would require additional API calls to get actual power level
        }))
    }

    /// Gets item name from cache or creates a placeholder
    /// 
    /// # Arguments
    /// * `item_hash` - The item hash to look up
    /// 
    /// # Returns
    /// * `String` - Item name or placeholder
    async fn get_item_name(&mut self, item_hash: u32) -> String {
        if let Some(name) = self.item_definitions.get(&item_hash) {
            name.clone()
        } else {
            // Create a placeholder name based on hash
            let placeholder = format!("Unknown Item ({})", item_hash);
            self.item_definitions.insert(item_hash, placeholder.clone());
            placeholder
        }
    }

    /// Determines weapon type based on item hash (simplified implementation)
    /// 
    /// # Arguments
    /// * `item_hash` - The weapon's item hash
    /// 
    /// # Returns
    /// * `String` - Weapon type classification
    async fn determine_weapon_type(&self, item_hash: u32) -> String {
        // This is a simplified implementation. In reality, you would need to
        // query Bungie's item definitions API to get the actual weapon type
        match item_hash % 10 {
            0 => "Auto Rifle".to_string(),
            1 => "Hand Cannon".to_string(),
            2 => "Pulse Rifle".to_string(),
            3 => "Scout Rifle".to_string(),
            4 => "Submachine Gun".to_string(),
            5 => "Shotgun".to_string(),
            6 => "Sniper Rifle".to_string(),
            7 => "Fusion Rifle".to_string(),
            8 => "Rocket Launcher".to_string(),
            _ => "Unknown Weapon".to_string(),
        }
    }

    /// Determines damage type based on item hash (simplified implementation)
    /// 
    /// # Arguments
    /// * `item_hash` - The weapon's item hash
    /// 
    /// # Returns
    /// * `String` - Damage type classification
    async fn determine_damage_type(&self, item_hash: u32) -> String {
        // This is a simplified implementation. In reality, you would need to
        // query Bungie's item definitions API to get the actual damage type
        match item_hash % 6 {
            0 => "Kinetic".to_string(),
            1 => "Solar".to_string(),
            2 => "Arc".to_string(),
            3 => "Void".to_string(),
            4 => "Stasis".to_string(),
            5 => "Strand".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Analyzes kill attribution for a participant
    /// 
    /// # Arguments
    /// * `participant` - The participant to analyze
    /// 
    /// # Returns
    /// * `Result<KillAttribution>` - Kill attribution analysis or an error
    async fn analyze_kill_attribution(
        &self,
        participant: &crate::models::ParticipantEntry,
    ) -> Result<KillAttribution> {
        let mut weapon_kills = 0;
        let mut weapon_breakdown = HashMap::new();
        let mut ability_breakdown = HashMap::new();
    
        // Main stats are in participant.values (not extended.values)
        let values = &participant.values;
        let extended_values = &participant.extended.values;
    
        // Get total kills from main values
        let total_kills = values.get("kills").map(|s| s.basic.value).unwrap_or(0.0) as i32;
        
        // Ability kills are in extended.values with different names
        let grenade_kills = extended_values.get("weaponKillsGrenade").map(|s| s.basic.value).unwrap_or(0.0) as i32;
        let melee_kills = extended_values.get("weaponKillsMelee").map(|s| s.basic.value).unwrap_or(0.0) as i32;
        let super_kills = extended_values.get("weaponKillsSuper").map(|s| s.basic.value).unwrap_or(0.0) as i32;
    
        // Analyze weapon kills from weapon stats (use uniqueWeaponKills)
        for weapon in &participant.extended.weapons {
            let weapon_kills_count = weapon.values.get("uniqueWeaponKills").map(|s| s.basic.value).unwrap_or(0.0) as i32;
            weapon_kills += weapon_kills_count;
            
            let weapon_name = format!("Weapon {}", weapon.reference_id);
            weapon_breakdown.insert(weapon_name, weapon_kills_count);
        }
    
        // Calculate ability kills
        let ability_kills = grenade_kills + melee_kills + super_kills;
        
        ability_breakdown.insert("Grenade".to_string(), grenade_kills);
        ability_breakdown.insert("Melee".to_string(), melee_kills);
        ability_breakdown.insert("Super".to_string(), super_kills);
    
        // Environmental kills
        let environmental_kills = total_kills - weapon_kills - ability_kills;
    
        Ok(KillAttribution {
            total_kills,
            weapon_kills,
            ability_kills,
            environmental_kills,
            weapon_breakdown,
            ability_breakdown,
        })
    }

    /// Calculates a performance score for the participant
    /// 
    /// # Arguments
    /// * `participant` - The participant to score
    /// 
    /// # Returns
    /// * `Result<f64>` - Performance score or an error
    async fn calculate_performance_score(
        &self,
        participant: &crate::models::ParticipantEntry,
    ) -> Result<f64> {
        let values = &participant.values;

        let kills = values.get("kills").map(|s| s.basic.value).unwrap_or(0.0);
        let deaths = values.get("deaths").map(|s| s.basic.value).unwrap_or(0.0);
        let assists = values.get("assists").map(|s| s.basic.value).unwrap_or(0.0);
        let score = participant.score.basic.value;

        let kd_ratio = if deaths > 0.0 { kills / deaths } else { kills };
        let performance_score = (kd_ratio * 0.4) + (score * 0.0001) + (assists * 0.1);

        Ok(performance_score)
    }

    /// Creates an activity summary for the carnage report
    /// 
    /// # Arguments
    /// * `report` - The carnage report to summarize
    /// 
    /// # Returns
    /// * `Result<ActivitySummary>` - Activity summary or an error
    async fn create_activity_summary(&self, report: &PostGameCarnageReport) -> Result<ActivitySummary> {
        // Determine activity type based on mode
        let activity_type = match report.activity_details.mode {
            2 => ActivityType::Story,
            3 => ActivityType::Strike,
            4 => ActivityType::Raid,
            5 => ActivityType::Crucible,
            6 => ActivityType::Gambit,
            7 => ActivityType::Dungeon,
            _ => ActivityType::Other,
        };

        // Calculate total fireteam statistics
        let mut total_kills = 0;
        let mut total_deaths = 0;
    
        for entry in &report.entries {
            total_kills += entry.values.get("kills").map(|s| s.basic.value).unwrap_or(0.0) as i32;
            total_deaths += entry.values.get("deaths").map(|s| s.basic.value).unwrap_or(0.0) as i32;
        }

        // Determine completion status (simplified - would need more sophisticated logic)
        let completion_status = if total_deaths == 0 { "Flawless" } else { "Completed" }.to_string();

        Ok(ActivitySummary {
            activity_type,
            duration: "Unknown".to_string(), // Would require additional API calls
            total_fireteam_kills: total_kills,
            total_fireteam_deaths: total_deaths,
            completion_status,
        })
    }
}

impl Default for CarnageReportAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
