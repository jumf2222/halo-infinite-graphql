use async_graphql::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchesResponse {
    pub count: i32,
    pub links: Value,
    pub result_count: i32,
    pub start: i32,
    pub results: Vec<Match>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Match {
    pub last_team_id: i32,
    pub match_id: String,
    pub match_info: MatchInfo,
    pub outcome: i32,
    pub present_at_end_of_match: bool,
    pub rank: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchInfo {
    pub clearance_id: String,
    pub duration: String,
    pub end_time: String,
    pub game_variant_category: i32,
    pub gameplay_interaction: i32,
    pub level_id: String,
    pub lifecycle_mode: i32,
    pub map_variant: AssetReference,
    pub playable_duration: String,
    pub playlist: Option<Value>,
    pub playlist_experience: Option<Value>,
    pub playlist_map_mode_pair: Option<Value>,
    pub season_id: Option<Value>,
    pub start_time: String,
    pub team_scoring_enabled: bool,
    pub teams_enabled: bool,
    pub ugc_game_variant: AssetReference,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AssetReference {
    pub asset_id: String,
    pub asset_kind: i32,
    pub version_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SkillResponse {
    value: Vec<Skill>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResultRankRecap {
    pub pre_match_csr: SkillResultRankRecapCsr,
    pub post_match_csr: SkillResultRankRecapCsr,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResultRankRecapCsr {
    pub value: i32,
    pub measurement_matches_remaining: i32,
    pub tier: String,
    pub tier_start: i32,
    pub sub_tier: i32,
    pub next_tier: String,
    pub next_tier_start: i32,
    pub next_sub_tier: i32,
    pub initial_measurement_matches: i32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResultStatPerformances {
    pub kills: Option<SkillResultStatPerformance>,
    pub deaths: Option<SkillResultStatPerformance>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResultStatPerformance {
    pub count: i32,
    pub expected: f32,
    pub std_dev: f32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResultCounterfactuals {
    pub self_counterfactuals: SkillResultCounterfactualsKillsDeaths,
    pub tier_counterfactuals: SkillResultCounterfactualsTierCounterfactuals,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResultCounterfactualsKillsDeaths {
    pub kills: f32,
    pub deaths: f32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResultCounterfactualsTierCounterfactuals {
    pub bronze: SkillResultCounterfactualsKillsDeaths,
    pub silver: SkillResultCounterfactualsKillsDeaths,
    pub gold: SkillResultCounterfactualsKillsDeaths,
    pub platinum: SkillResultCounterfactualsKillsDeaths,
    pub diamond: SkillResultCounterfactualsKillsDeaths,
    pub onyx: SkillResultCounterfactualsKillsDeaths,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkillResult {
    pub team_mmr: f32,
    pub rank_recap: SkillResultRankRecap,
    pub stat_performances: Option<SkillResultStatPerformances>,
    pub team_id: i32,
    pub team_mmrs: HashMap<String, f32>,
    pub ranked_rewards: Option<Value>,
    pub counterfactuals: SkillResultCounterfactuals,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Skill {
    pub id: String,
    pub result_code: i32,
    pub result: SkillResult,
}

#[derive(Serialize, Deserialize)]
pub struct Gamer {
    pub xuid: String,
    pub gamertag: String,
    pub gamerpic: GamerPic,
}

#[derive(Serialize, Deserialize)]
pub struct GamerPic {
    pub small: String,
    pub medium: String,
    pub large: String,
    pub xlarge: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsTeam {
    pub team_id: i32,
    pub outcome: i32,
    pub rank: i32,
    pub stats: MatchStatsTeamStats,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsTeamStatsCoreStatsScore {
    pub name_id: i64,
    pub count: i32,
    pub total_personal_score_awarded: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsTeamStatsCoreStats {
    pub score: i32,
    pub personal_score: i32,
    pub rounds_won: i32,
    pub rounds_lost: i32,
    pub rounds_tied: i32,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    #[serde(rename = "KDA")]
    pub kda: f32,
    pub suicides: i32,
    pub betrayals: i32,
    pub average_life_duration: String,
    pub grenade_kills: i32,
    pub headshot_kills: i32,
    pub melee_kills: i32,
    pub power_weapon_kills: i32,
    pub shots_fired: i32,
    pub shots_hit: i32,
    pub accuracy: f32,
    pub damage_dealt: i32,
    pub damage_taken: i32,
    pub callout_assists: i32,
    pub vehicle_destroys: i32,
    pub driver_assists: i32,
    pub hijacks: i32,
    pub emp_assists: i32,
    pub max_killing_spree: i32,
    pub medals: Vec<MatchStatsTeamStatsCoreStatsScore>,
    pub personal_scores: Vec<MatchStatsTeamStatsCoreStatsScore>,
    pub deprecated_damage_dealt: f32,
    pub deprecated_damage_taken: f32,
    pub spawns: i32,
    pub objectives_completed: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsTeamStatsZonesStats {
    pub stronghold_captures: i32,
    pub stronghold_defensive_kills: i32,
    pub stronghold_offensive_kills: i32,
    pub stronghold_secures: i32,
    pub stronghold_occupation_time: String,
    pub stronghold_scoring_ticks: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsTeamStats {
    pub core_stats: MatchStatsTeamStatsCoreStats,
    pub zones_stats: Option<MatchStatsTeamStatsZonesStats>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsPlayerParticipationInfo {
    pub first_joined_time: String,
    pub last_leave_time: Option<String>,
    pub present_at_beginning: bool,
    pub joined_in_progress: bool,
    pub left_in_progress: bool,
    pub present_at_completion: bool,
    pub time_played: String,
    pub confirmed_participation: Option<Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsPlayer {
    pub player_id: String,
    pub player_type: i32,
    pub bot_attributes: Option<Value>,
    pub last_team_id: i32,
    pub outcome: i32,
    pub rank: i32,
    pub participation_info: MatchStatsPlayerParticipationInfo,
    /// Stats for the player when playing on each team
    pub player_team_stats: Vec<MatchStatsPlayerPlayerTeamStat>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStatsPlayerPlayerTeamStat {
    pub team_id: i32,
    pub stats: MatchStatsTeamStats,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchStats {
    pub match_id: String,
    pub match_info: MatchInfo,
    pub teams: Vec<MatchStatsTeam>,
    pub players: Vec<MatchStatsPlayer>,
}

pub async fn matches(
    client: &Client,
    spartan_token: &str,
    xuid: &str,
    start: Option<usize>,
    count: Option<usize>,
) -> Result<MatchesResponse> {
    Ok(client
        .get(format!(
            "https://halostats.svc.halowaypoint.com/hi/players/xuid({xuid})/matches"
        ))
        .query(&[("start", start), ("count", count)])
        .header("x-343-authorization-spartan", spartan_token)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<MatchesResponse>()
        .await?)
}

pub async fn gamer(client: &Client, spartan_token: &str, gamertag: &str) -> Result<Gamer> {
    Ok(client
        .get(format!(
            "https://profile.svc.halowaypoint.com/users/gt({gamertag})"
        ))
        .header("x-343-authorization-spartan", spartan_token)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<Gamer>()
        .await?)
}

pub async fn skill(
    client: &Client,
    spartan_token: &str,
    match_id: &str,
    xuids: &[String],
) -> Result<Vec<Skill>> {
    println!(
        "URL: {}",
        format!(
            "https://skill.svc.halowaypoint.com/hi/matches/{match_id}/skill?players={}",
            xuids
                .iter()
                .map(|x| format!("xuid({}),", x))
                .collect::<String>()
                .trim_end_matches(",")
        )
    );
    Ok(client
        .get(format!(
            "https://skill.svc.halowaypoint.com/hi/matches/{match_id}/skill?players={}",
            xuids
                .iter()
                .map(|x| format!("xuid({}),", x))
                .collect::<String>()
                .trim_end_matches(",")
        ))
        .header("x-343-authorization-spartan", spartan_token)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<SkillResponse>()
        .await?
        .value)
}

pub async fn stats(client: &Client, spartan_token: &str, match_id: &str) -> Result<MatchStats> {
    Ok(client
        .get(format!(
            "https://halostats.svc.halowaypoint.com/hi/matches/{match_id}/stats",
        ))
        .header("x-343-authorization-spartan", spartan_token)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<MatchStats>()
        .await?)
}
