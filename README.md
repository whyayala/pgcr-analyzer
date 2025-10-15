# Bungie Post-Game Carnage Report Analyzer

A comprehensive Rust application for analyzing Destiny 2 post-game carnage reports through the Bungie API. This tool allows you to query users by their Bungie ID, analyze their recent activities, and examine detailed equipment and kill attribution data for both individual players and their fireteams.

## Features

- 🔍 **User Search**: Find players by their Bungie display name
- 📊 **Activity Analysis**: Analyze recent activities with filtering by activity type
- 🎯 **Equipment Tracking**: Extract detailed equipment information for users and fireteam members
- ⚔️ **Kill Attribution**: Distinguish between weapon kills, ability kills, and environmental kills
- 👥 **Fireteam Analysis**: Comprehensive analysis of all fireteam members' performance
- 📋 **Multiple Output Formats**: JSON, table, and detailed text output formats
- 🚦 **Rate Limiting**: Built-in rate limiting to respect Bungie API limits
- 🛡️ **Error Handling**: Comprehensive error handling with helpful error messages

## Prerequisites

- Rust 1.70+ (2021 edition)
- A Bungie API key (free from [Bungie Developer Portal](https://www.bungie.net/en/Application))

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd bungie-pgcr-analyzer
```

2. Set up your Bungie API key:
```bash
export BUNGIE_API_KEY=your_api_key_here
```

3. Build the application:
```bash
cargo build --release
```

## Usage

### Command Line Interface

The application provides a command-line interface with the following commands:

#### Search for Users
```bash
bungie-analyzer search --name "PlayerDisplayName"
```

#### Analyze Recent Activities
```bash
# Analyze recent Crucible matches for a Steam user
bungie-analyzer analyze --user-id "123456789" --membership-type 4 --activity-type crucible --count 10

# Analyze recent Raids with table output
bungie-analyzer analyze --user-id "123456789" --membership-type 4 --activity-type raid --count 5 --format table
```

#### Analyze Specific Activity
```bash
bungie-analyzer activity --activity-id "1234567890" --user-id "123456789" --format detailed
```

### Membership Types

- `1` - Xbox
- `2` - PlayStation
- `4` - Steam
- `10` - Battle.net
- `254` - Bungie

### Activity Types

- `story` - Story missions and campaigns
- `strike` - Strikes and nightfalls
- `crucible` - Crucible PvP matches
- `gambit` - Gambit matches
- `raid` - Raid encounters
- `dungeon` - Dungeon encounters
- `other` - All other activity types

### Output Formats

- `table` - Clean table format (default for analyze command)
- `detailed` - Comprehensive detailed format (default for activity command)
- `json` - JSON format for programmatic processing

## Example Output

### Table Format
```
=== FIRETEAM ANALYSIS ===
Activity Type: Crucible
Completion Status: Completed
Total Fireteam Kills: 45
Total Fireteam Deaths: 32

--- Primary User ---
  Guardian123 (Warlock): K/D 1.25, Score 2850, Performance 2.1

--- Fireteam Members ---
  Teammate1 (Hunter): K/D 0.89, Score 2100, Performance 1.5
  Teammate2 (Titan): K/D 1.45, Score 3200, Performance 2.8
```

### Detailed Format
```
=== DETAILED FIRETEAM ANALYSIS ===

ACTIVITY SUMMARY:
  Type: Crucible
  Duration: 12:34
  Status: Completed
  Total Kills: 45
  Total Deaths: 32

PRIMARY USER:
    Name: Guardian123
    Class: Warlock
    Level: 40
    Light Level: 1810
    Performance Score: 2.1
    
    KILL ATTRIBUTION:
      Total Kills: 15
      Weapon Kills: 12
      Ability Kills: 3
      Environmental Kills: 0
      
      Weapon Breakdown:
        Hand Cannon: 8
        Fusion Rifle: 4
        
      Ability Breakdown:
        Grenade: 2
        Melee: 1
        
    EQUIPMENT:
      Primary: Ace of Spades (Hand Cannon)
      Secondary: Main Ingredient (Fusion Rifle)
      Heavy: Gjallarhorn (Rocket Launcher)
```

## Library Usage

You can also use this as a library in your own Rust projects:

```rust
use bungie_pgcr_analyzer::api_client::create_client_from_env;
use bungie_pgcr_analyzer::analyzer::CarnageReportAnalyzer;
use bungie_pgcr_analyzer::models::ActivityType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create API client
    let client = create_client_from_env()?;
    
    // Search for a user
    let search_results = client.search_users("PlayerName").await?;
    
    // Get their recent activities
    let reports = client.get_post_game_carnage_reports(
        4, // Steam membership type
        &search_results.search_results[0].membership_id,
        Some(ActivityType::Crucible),
        Some(10)
    ).await?;
    
    // Analyze the reports
    let mut analyzer = CarnageReportAnalyzer::new();
    for report in reports {
        let analysis = analyzer.analyze_report(&report, &user_id).await?;
        println!("Analysis: {:?}", analysis);
    }
    
    Ok(())
}
```

## Development

### Code Quality

This project follows Rust best practices and includes:

- **Clippy**: Additional lints for code quality (see `clippy.toml`)
- **Rustfmt**: Consistent code formatting (see `rustfmt.toml`)
- **Comprehensive Documentation**: Inline documentation for all public APIs
- **Error Handling**: Custom error types with helpful error messages
- **Async/Await**: Modern async Rust patterns with tokio

### Running Tests

```bash
cargo test
```

### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with pedantic lints
cargo clippy -- -W clippy::pedantic

# Format code
cargo fmt
```

### Project Structure

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library exports and documentation
├── api_client.rs    # Bungie API client implementation
├── models.rs        # Data models for API responses
├── analyzer.rs      # Carnage report analysis logic
├── cli.rs           # Command-line interface
└── error.rs         # Custom error types
```

## API Rate Limiting

The application includes built-in rate limiting to respect Bungie's API limits. By default, it includes a 100ms delay between requests. You can configure this in the `BungieApiConfig` structure.

## Error Handling

The application uses a comprehensive error handling system with custom error types:

- `BungieError::ApiError` - Bungie API specific errors
- `BungieError::UserNotFound` - User not found errors
- `BungieError::RateLimitExceeded` - Rate limiting errors
- `BungieError::AuthenticationError` - Authentication failures
- And more...

All errors include helpful messages and context to assist with debugging.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure your code follows the project's formatting and linting standards.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Bungie](https://www.bungie.net/) for providing the Destiny 2 API
- The Rust community for excellent documentation and tooling
- Destiny 2 community for inspiration and feedback

## Disclaimer

This project is not affiliated with Bungie or Destiny 2. It's a community tool for analyzing game data through the official Bungie API. Please respect Bungie's Terms of Service and API usage guidelines.
