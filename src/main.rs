use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};
use async_graphql::dataloader::*;
use async_graphql::types::connection::*;
use async_graphql::OutputType;
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Result,
    Schema, SimpleObject,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use dotenv::dotenv;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

mod auth;
mod halo_requests;

struct Query;

#[derive(SimpleObject)]
struct SpartanToken {
    token: String,
    expires_at: String,
    refresh_token: String,
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct Player {
    id: String,
    gamertag: String,
    pic: PlayerPic,
}

#[derive(SimpleObject)]
struct PlayerPic {
    small: String,
    medium: String,
    large: String,
    xlarge: String,
}

#[derive(SimpleObject)]
struct AssetReference {
    asset_id: String,
    asset_kind: i32,
    version_id: String,
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct Match {
    id: String,
    clearance_id: String,
    duration: String,
    end_time: String,
    game_variant_category: i32,
    gameplay_interaction: i32,
    level_id: String,
    lifecycle_mode: i32,
    map_variant: AssetReference,
    playable_duration: String,
    playlist: Option<Value>,
    playlist_experience: Option<Value>,
    playlist_map_mode_pair: Option<Value>,
    season_id: Option<Value>,
    start_time: String,
    team_scoring_enabled: bool,
    teams_enabled: bool,
    ugc_game_variant: AssetReference,
}

#[derive(SimpleObject)]
struct MatchEdgeData {
    last_team_id: i32,
    outcome: i32,
    present_at_end_of_match: bool,
    rank: i32,
}

#[derive(SimpleObject)]
struct Team {
    team_id: i32,
    rank: i32,
    players: Connection<
        usize,
        Option<Player>,
        EmptyFields,
        TeamPlayerEdgeData,
        TeamPlayerConnection,
        TeamPlayerEdge,
    >,
}

#[derive(SimpleObject)]
struct TeamEdgeData {
    outcome: i32,
    score: i32,
    total_personal_score: i32,
    rounds_won: i32,
    rounds_lost: i32,
    rounds_tied: i32,
    kills: i32,
    deaths: i32,
    assists: i32,
    kda: f32,
    suicides: i32,
    betrayals: i32,
    average_life_duration: String,
    grenade_kills: i32,
    headshot_kills: i32,
    melee_kills: i32,
    power_weapon_kills: i32,
    shots_fired: i32,
    shots_hit: i32,
    accuracy: f32,
    damage_dealt: i32,
    damage_taken: i32,
    callout_assists: i32,
    vehicle_destroys: i32,
    driver_assists: i32,
    hijacks: i32,
    emp_assists: i32,
    max_killing_spree: i32,
    medals: Vec<ScoreChange>,
    personal_scores: Vec<ScoreChange>,
    deprecated_damage_dealt: f32,
    deprecated_damage_taken: f32,
    spawns: i32,
    objectives_completed: i32,
    stronghold_stats: Option<StrongholdStats>,
}

#[derive(SimpleObject)]
struct StrongholdStats {
    captures: i32,
    defensive_kills: i32,
    offensive_kills: i32,
    secures: i32,
    occupation_time: String,
    scoring_ticks: i32,
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct PlayerEdgeData {
    match_id: String,
    player_id: String,
    player_type: i32,
    bot_attributes: Option<Value>,
    last_team_id: i32,
    outcome: i32,
    rank: i32,
    first_joined_time: String,
    last_leave_time: Option<String>,
    present_at_beginning: bool,
    joined_in_progress: bool,
    left_in_progress: bool,
    present_at_completion: bool,
    time_played: String,
    confirmed_participation: Option<Value>,
}

#[derive(SimpleObject)]
struct Csr {
    value: i32,
    measurement_matches_remaining: i32,
    tier: String,
    tier_start: i32,
    sub_tier: i32,
    next_tier: String,
    next_tier_start: i32,
    next_sub_tier: i32,
    initial_measurement_matches: i32,
}

#[derive(SimpleObject)]
struct TeamPlayerEdgeData {
    player_id: String,
    score: i32,
    personal_score: i32,
    rounds_won: i32,
    rounds_lost: i32,
    rounds_tied: i32,
    kills: i32,
    deaths: i32,
    assists: i32,
    kda: f32,
    suicides: i32,
    betrayals: i32,
    average_life_duration: String,
    grenade_kills: i32,
    headshot_kills: i32,
    melee_kills: i32,
    power_weapon_kills: i32,
    shots_fired: i32,
    shots_hit: i32,
    accuracy: f32,
    damage_dealt: i32,
    damage_taken: i32,
    callout_assists: i32,
    vehicle_destroys: i32,
    driver_assists: i32,
    hijacks: i32,
    emp_assists: i32,
    max_killing_spree: i32,
    medals: Vec<ScoreChange>,
    personal_scores: Vec<ScoreChange>,
    deprecated_damage_dealt: f32,
    deprecated_damage_taken: f32,
    spawns: i32,
    objectives_completed: i32,
    stronghold_stats: Option<StrongholdStats>,
}

struct TeamPlayerConnection;

impl ConnectionNameType for TeamPlayerConnection {
    fn type_name<T: OutputType>() -> String {
        "TeamPlayerConnection".to_string()
    }
}
struct TeamPlayerEdge;

impl EdgeNameType for TeamPlayerEdge {
    fn type_name<T: OutputType>() -> String {
        "TeamPlayerEdge".to_string()
    }
}

#[derive(SimpleObject)]
struct ScoreChange {
    name_id: i64,
    count: i32,
    total_personal_score_awarded: i32,
}

#[ComplexObject]
impl PlayerEdgeData {
    async fn pre_match_csr<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Csr> {
        let data = ctx.data_unchecked::<AuthData>();

        data.loader
            .load_one(SkillEntry {
                player_id: self
                    .player_id
                    .trim_start_matches("xuid(")
                    .trim_end_matches(")")
                    .to_string(),
                match_id: self.match_id.clone(),
            })
            .await?
            .ok_or(async_graphql::Error::new("Failed to fetch"))
            .map(|res| Csr {
                value: res.result.rank_recap.pre_match_csr.value,
                measurement_matches_remaining: res
                    .result
                    .rank_recap
                    .pre_match_csr
                    .measurement_matches_remaining,
                tier: res.result.rank_recap.pre_match_csr.tier,
                tier_start: res.result.rank_recap.pre_match_csr.tier_start,
                sub_tier: res.result.rank_recap.pre_match_csr.sub_tier,
                next_tier: res.result.rank_recap.pre_match_csr.next_tier,
                next_tier_start: res.result.rank_recap.pre_match_csr.next_tier_start,
                next_sub_tier: res.result.rank_recap.pre_match_csr.next_sub_tier,
                initial_measurement_matches: res
                    .result
                    .rank_recap
                    .pre_match_csr
                    .initial_measurement_matches,
            })
    }

    async fn post_match_csr<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Csr> {
        let data = ctx.data_unchecked::<AuthData>();

        data.loader
            .load_one(SkillEntry {
                player_id: self
                    .player_id
                    .trim_start_matches("xuid(")
                    .trim_end_matches(")")
                    .to_string(),
                match_id: self.match_id.clone(),
            })
            .await?
            .ok_or(async_graphql::Error::new("Failed to fetch"))
            .map(|res| Csr {
                value: res.result.rank_recap.post_match_csr.value,
                measurement_matches_remaining: res
                    .result
                    .rank_recap
                    .post_match_csr
                    .measurement_matches_remaining,
                tier: res.result.rank_recap.post_match_csr.tier,
                tier_start: res.result.rank_recap.post_match_csr.tier_start,
                sub_tier: res.result.rank_recap.post_match_csr.sub_tier,
                next_tier: res.result.rank_recap.post_match_csr.next_tier,
                next_tier_start: res.result.rank_recap.post_match_csr.next_tier_start,
                next_sub_tier: res.result.rank_recap.post_match_csr.next_sub_tier,
                initial_measurement_matches: res
                    .result
                    .rank_recap
                    .post_match_csr
                    .initial_measurement_matches,
            })
    }

    async fn expected_kills<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<f32>> {
        let data = ctx.data_unchecked::<AuthData>();

        data.loader
            .load_one(SkillEntry {
                player_id: self
                    .player_id
                    .trim_start_matches("xuid(")
                    .trim_end_matches(")")
                    .to_string(),
                match_id: self.match_id.clone(),
            })
            .await?
            .ok_or(async_graphql::Error::new("Failed to fetch"))
            .map(|res| {
                res.result
                    .stat_performances
                    .and_then(|x| x.kills)
                    .map(|x| x.expected)
            })
    }

    async fn expected_deaths<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<f32>> {
        let data = ctx.data_unchecked::<AuthData>();

        data.loader
            .load_one(SkillEntry {
                player_id: self
                    .player_id
                    .trim_start_matches("xuid(")
                    .trim_end_matches(")")
                    .to_string(),
                match_id: self.match_id.clone(),
            })
            .await?
            .ok_or(async_graphql::Error::new("Failed to fetch"))
            .map(|res| {
                res.result
                    .stat_performances
                    .and_then(|x| x.deaths)
                    .map(|x| x.expected)
            })
    }
}

#[ComplexObject]
impl Player {
    async fn matches<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Match, EmptyFields, MatchEdgeData>> {
        query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let data = ctx.data_unchecked::<AuthData>();

                let first = first.map(|x| x.min(24));
                let last = last.map(|x| x.min(24));
                let mut start = after.map(|after| after + 1).unwrap_or(0);
                let mut end = before.unwrap_or(10000);

                if let Some(first) = first {
                    end = (start + first).min(end);
                }

                if let Some(last) = last {
                    start = if last > end - start {
                        start
                    } else {
                        end - last
                    };
                }

                let fetch_count = (end - start + 1).min(25);

                let res = halo_requests::matches(
                    &data.client,
                    &data.spartan_token,
                    &self.id,
                    Some(start),
                    Some(fetch_count),
                )
                .await?;

                let mut connection = Connection::new(start > 0, res.results.len() == fetch_count);

                connection.edges.extend(
                    res.results
                        .into_iter()
                        .take(fetch_count - 1)
                        .enumerate()
                        .map(|(ind, x)| {
                            Edge::with_additional_fields(
                                start + ind,
                                Match {
                                    id: x.match_id,
                                    clearance_id: x.match_info.clearance_id,
                                    duration: x.match_info.duration,
                                    end_time: x.match_info.end_time,
                                    game_variant_category: x.match_info.game_variant_category,
                                    gameplay_interaction: x.match_info.gameplay_interaction,
                                    level_id: x.match_info.level_id,
                                    lifecycle_mode: x.match_info.lifecycle_mode,
                                    map_variant: AssetReference {
                                        asset_id: x.match_info.map_variant.asset_id,
                                        asset_kind: x.match_info.map_variant.asset_kind,
                                        version_id: x.match_info.map_variant.version_id,
                                    },
                                    playable_duration: x.match_info.playable_duration,
                                    playlist: x.match_info.playlist,
                                    playlist_experience: x.match_info.playlist_experience,
                                    playlist_map_mode_pair: x.match_info.playlist_map_mode_pair,
                                    season_id: x.match_info.season_id,
                                    start_time: x.match_info.start_time,
                                    team_scoring_enabled: x.match_info.team_scoring_enabled,
                                    teams_enabled: x.match_info.teams_enabled,
                                    ugc_game_variant: AssetReference {
                                        asset_id: x.match_info.ugc_game_variant.asset_id,
                                        asset_kind: x.match_info.ugc_game_variant.asset_kind,
                                        version_id: x.match_info.ugc_game_variant.version_id,
                                    },
                                },
                                MatchEdgeData {
                                    last_team_id: x.last_team_id,
                                    outcome: x.outcome,
                                    present_at_end_of_match: x.present_at_end_of_match,
                                    rank: x.rank,
                                },
                            )
                        }),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}

// #[ComplexObject]
// impl Match {
//     async fn stats<'ctx>(&self, ctx: &Context<'ctx>) -> Result<MatchStats> {
//         let data = ctx.data::<AuthData>().unwrap();

//         Ok(data
//             .client
//             .get(format!(
//                 "https://halostats.svc.halowaypoint.com/hi/matches/{}/stats",
//                 self.match_id
//             ))
//             .header("x-343-authorization-spartan", data.spartan_token.as_str())
//             .header("Accept", "application/json")
//             .send()
//             .await?
//             .json::<MatchStats>()
//             .await?)
//     }
// }

#[ComplexObject]
impl Match {
    async fn teams<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Connection<usize, Team, EmptyFields, TeamEdgeData>> {
        let data = ctx.data_unchecked::<AuthData>();

        let res = halo_requests::stats(&data.client, &data.spartan_token, &self.id).await?;

        let mut connection = Connection::new(false, false);

        connection
            .edges
            .extend(res.teams.into_iter().enumerate().map(|(ind, x)| {
                let mut player_connection = Connection::new(false, false);

                player_connection
                    .edges
                    .extend(res.players.iter().flat_map(|y| {
                        let stats = y.player_team_stats.iter().find(|y| y.team_id == x.team_id);

                        match stats {
                            Some(x) => Some(Edge::with_additional_fields(
                                0,
                                None,
                                TeamPlayerEdgeData {
                                    player_id: y.player_id.clone(),
                                    score: x.stats.core_stats.score,
                                    personal_score: x.stats.core_stats.personal_score,
                                    rounds_won: x.stats.core_stats.rounds_won,
                                    rounds_lost: x.stats.core_stats.rounds_lost,
                                    rounds_tied: x.stats.core_stats.rounds_tied,
                                    kills: x.stats.core_stats.kills,
                                    deaths: x.stats.core_stats.deaths,
                                    assists: x.stats.core_stats.assists,
                                    kda: x.stats.core_stats.kda,
                                    suicides: x.stats.core_stats.suicides,
                                    betrayals: x.stats.core_stats.betrayals,
                                    average_life_duration: x
                                        .stats
                                        .core_stats
                                        .average_life_duration
                                        .clone(),
                                    grenade_kills: x.stats.core_stats.grenade_kills,
                                    headshot_kills: x.stats.core_stats.headshot_kills,
                                    melee_kills: x.stats.core_stats.melee_kills,
                                    power_weapon_kills: x.stats.core_stats.power_weapon_kills,
                                    shots_fired: x.stats.core_stats.shots_fired,
                                    shots_hit: x.stats.core_stats.shots_hit,
                                    accuracy: x.stats.core_stats.accuracy,
                                    damage_dealt: x.stats.core_stats.damage_dealt,
                                    damage_taken: x.stats.core_stats.damage_taken,
                                    callout_assists: x.stats.core_stats.callout_assists,
                                    vehicle_destroys: x.stats.core_stats.vehicle_destroys,
                                    driver_assists: x.stats.core_stats.driver_assists,
                                    hijacks: x.stats.core_stats.hijacks,
                                    emp_assists: x.stats.core_stats.emp_assists,
                                    max_killing_spree: x.stats.core_stats.max_killing_spree,
                                    medals: x
                                        .stats
                                        .core_stats
                                        .medals
                                        .iter()
                                        .map(|x| ScoreChange {
                                            count: x.count,
                                            total_personal_score_awarded: x
                                                .total_personal_score_awarded,
                                            name_id: x.name_id,
                                        })
                                        .collect(),
                                    personal_scores: x
                                        .stats
                                        .core_stats
                                        .personal_scores
                                        .iter()
                                        .map(|x| ScoreChange {
                                            count: x.count,
                                            total_personal_score_awarded: x
                                                .total_personal_score_awarded,
                                            name_id: x.name_id,
                                        })
                                        .collect(),
                                    deprecated_damage_dealt: x
                                        .stats
                                        .core_stats
                                        .deprecated_damage_dealt,
                                    deprecated_damage_taken: x
                                        .stats
                                        .core_stats
                                        .deprecated_damage_taken,
                                    spawns: x.stats.core_stats.spawns,
                                    objectives_completed: x.stats.core_stats.objectives_completed,
                                    stronghold_stats: x.stats.zones_stats.as_ref().map(|x| {
                                        StrongholdStats {
                                            captures: x.stronghold_captures,
                                            defensive_kills: x.stronghold_defensive_kills,
                                            offensive_kills: x.stronghold_offensive_kills,
                                            secures: x.stronghold_secures,
                                            occupation_time: x.stronghold_occupation_time.clone(),
                                            scoring_ticks: x.stronghold_scoring_ticks,
                                        }
                                    }),
                                },
                            )),
                            _ => None,
                        }
                    }));

                Edge::with_additional_fields(
                    ind,
                    Team {
                        team_id: x.team_id,
                        rank: x.rank,
                        players: player_connection,
                    },
                    TeamEdgeData {
                        outcome: x.outcome,
                        score: x.stats.core_stats.score,
                        total_personal_score: x.stats.core_stats.personal_score,
                        rounds_won: x.stats.core_stats.rounds_won,
                        rounds_lost: x.stats.core_stats.rounds_lost,
                        rounds_tied: x.stats.core_stats.rounds_tied,
                        kills: x.stats.core_stats.kills,
                        deaths: x.stats.core_stats.deaths,
                        assists: x.stats.core_stats.assists,
                        kda: x.stats.core_stats.kda,
                        suicides: x.stats.core_stats.suicides,
                        betrayals: x.stats.core_stats.betrayals,
                        average_life_duration: x.stats.core_stats.average_life_duration,
                        grenade_kills: x.stats.core_stats.grenade_kills,
                        headshot_kills: x.stats.core_stats.headshot_kills,
                        melee_kills: x.stats.core_stats.melee_kills,
                        power_weapon_kills: x.stats.core_stats.power_weapon_kills,
                        shots_fired: x.stats.core_stats.shots_fired,
                        shots_hit: x.stats.core_stats.shots_hit,
                        accuracy: x.stats.core_stats.accuracy,
                        damage_dealt: x.stats.core_stats.damage_dealt,
                        damage_taken: x.stats.core_stats.damage_taken,
                        callout_assists: x.stats.core_stats.callout_assists,
                        vehicle_destroys: x.stats.core_stats.vehicle_destroys,
                        driver_assists: x.stats.core_stats.driver_assists,
                        hijacks: x.stats.core_stats.hijacks,
                        emp_assists: x.stats.core_stats.emp_assists,
                        max_killing_spree: x.stats.core_stats.max_killing_spree,
                        medals: x
                            .stats
                            .core_stats
                            .medals
                            .into_iter()
                            .map(|x| ScoreChange {
                                count: x.count,
                                total_personal_score_awarded: x.total_personal_score_awarded,
                                name_id: x.name_id,
                            })
                            .collect(),
                        personal_scores: x
                            .stats
                            .core_stats
                            .personal_scores
                            .into_iter()
                            .map(|x| ScoreChange {
                                count: x.count,
                                total_personal_score_awarded: x.total_personal_score_awarded,
                                name_id: x.name_id,
                            })
                            .collect(),
                        deprecated_damage_dealt: x.stats.core_stats.deprecated_damage_dealt,
                        deprecated_damage_taken: x.stats.core_stats.deprecated_damage_taken,
                        spawns: x.stats.core_stats.spawns,
                        objectives_completed: x.stats.core_stats.objectives_completed,
                        stronghold_stats: x.stats.zones_stats.map(|x| StrongholdStats {
                            captures: x.stronghold_captures,
                            defensive_kills: x.stronghold_defensive_kills,
                            offensive_kills: x.stronghold_offensive_kills,
                            secures: x.stronghold_secures,
                            occupation_time: x.stronghold_occupation_time,
                            scoring_ticks: x.stronghold_scoring_ticks,
                        }),
                    },
                )
            }));

        Ok(connection)
    }

    async fn players<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Connection<usize, Option<Player>, EmptyFields, PlayerEdgeData>> {
        let data = ctx.data_unchecked::<AuthData>();

        // data.loader.load_one(10 as f32).await

        let res = halo_requests::stats(&data.client, &data.spartan_token, &self.id).await?;

        let mut connection = Connection::new(false, false);

        connection
            .edges
            .extend(res.players.into_iter().enumerate().map(|(ind, x)| {
                Edge::with_additional_fields(
                    ind,
                    None,
                    PlayerEdgeData {
                        match_id: self.id.clone(),
                        player_id: x.player_id,
                        player_type: x.player_type,
                        bot_attributes: x.bot_attributes,
                        last_team_id: x.last_team_id,
                        outcome: x.outcome,
                        rank: x.rank,
                        first_joined_time: x.participation_info.first_joined_time,
                        last_leave_time: x.participation_info.last_leave_time,
                        present_at_beginning: x.participation_info.present_at_beginning,
                        joined_in_progress: x.participation_info.joined_in_progress,
                        left_in_progress: x.participation_info.left_in_progress,
                        present_at_completion: x.participation_info.present_at_completion,
                        time_played: x.participation_info.time_played,
                        confirmed_participation: x.participation_info.confirmed_participation,
                    },
                )
            }));

        Ok(connection)
    }
}

pub struct HaloLoader {
    pub client: Client,
    pub spartan_token: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct SkillEntry {
    player_id: String,
    match_id: String,
}

#[async_trait::async_trait]
impl Loader<SkillEntry> for HaloLoader {
    type Value = halo_requests::Skill;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[SkillEntry]) -> Result<HashMap<SkillEntry, Self::Value>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for x in keys.iter() {
            map.entry(x.match_id.clone())
                .or_default()
                .push(x.player_id.clone());
        }

        let futures: futures::stream::FuturesUnordered<_> = map
            .into_iter()
            .map(|(match_id, players)| async move {
                halo_requests::skill(&self.client, &self.spartan_token, &match_id, &players)
                    .await
                    .map_or_else(
                        |_| Vec::new(),
                        |x| {
                            x.into_iter()
                                .enumerate()
                                .map(|(ind, skill)| {
                                    (
                                        SkillEntry {
                                            player_id: players[ind].clone(),
                                            match_id: match_id.clone(),
                                        },
                                        skill,
                                    )
                                })
                                .collect()
                        },
                    )
            })
            .collect();

        let results: Vec<_> = futures.collect().await;

        Ok(results.into_iter().flat_map(|x| x.into_iter()).collect())
    }
}

#[Object]
impl Query {
    /// Spartan token
    async fn spartan_token<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "OAuth code.")] code: Option<String>,
        #[graphql(desc = "OAuth refresh token.")] refresh_token: Option<String>,
    ) -> Result<SpartanToken> {
        let data = ctx.data_unchecked::<AuthData>();

        let auth_token = auth::auth_token(&data.client, code, refresh_token).await?;
        let user_token = auth::user_token(&data.client, auth_token.access_token).await?;
        let xsts_token = auth::xsts_token(&data.client, user_token.token)
            .await?
            .token;
        let spartan_token = auth::spartan_token(&data.client, xsts_token).await?;

        Ok(SpartanToken {
            token: spartan_token.spartan_token,
            expires_at: spartan_token.expires_utc.iso8601_date,
            refresh_token: auth_token.refresh_token,
        })
    }

    /// OAuth redirect url
    async fn redirect_url(&self) -> String {
        auth::redirect_url()
    }

    // async fn matches<'ctx>(
    //     &self,
    //     ctx: &Context<'ctx>,
    //     xuid: String,
    //     start: Option<i32>,
    //     count: Option<i32>,
    // ) -> Result<MatchesResponse> {
    //     let data = ctx.data::<AuthData>().unwrap();

    //     Ok(data
    //         .client
    //         .get(format!(
    //             "https://halostats.svc.halowaypoint.com/hi/players/xuid({xuid})/matches"
    //         ))
    //         .query(&[("start", start), ("count", count)])
    //         .header("x-343-authorization-spartan", data.spartan_token.as_str())
    //         .header("Accept", "application/json")
    //         .send()
    //         .await?
    //         .json::<MatchesResponse>()
    //         .await?)
    // }

    // async fn match_stats<'ctx>(&self, ctx: &Context<'ctx>, match_id: String) -> Result<MatchStats> {
    //     let data = ctx.data::<AuthData>().unwrap();

    //     Ok(data
    //         .client
    //         .get(format!(
    //             "https://halostats.svc.halowaypoint.com/hi/matches/{match_id}/stats"
    //         ))
    //         .header("x-343-authorization-spartan", data.spartan_token.as_str())
    //         .header("Accept", "application/json")
    //         .send()
    //         .await?
    //         .json::<MatchStats>()
    //         .await?)
    // }

    // async fn skill<'ctx>(
    //     &self,
    //     ctx: &Context<'ctx>,
    //     xuid: String,
    //     match_id: String,
    // ) -> Result<Skill> {
    //     let data = ctx.data::<AuthData>().unwrap();

    //     Ok(data.client
    //         .get(format!(
    //             "https://skill.svc.halowaypoint.com/hi/matches/{match_id}/skill?players=xuid({xuid})"
    //         ))
    //         .header("x-343-authorization-spartan", data.spartan_token.as_str())
    //         .header("Accept", "application/json")
    //         .send()
    //         .await?
    //         .json::<SkillResponse>()
    //         .await?.value.pop().unwrap())
    // }

    async fn player<'ctx>(&self, ctx: &Context<'ctx>, gamertag: String) -> Result<Player> {
        let data = ctx.data::<AuthData>().unwrap();

        halo_requests::gamer(&data.client, &data.spartan_token, &gamertag)
            .await
            .map(|x| Player {
                id: x.xuid,
                gamertag: x.gamertag,
                pic: PlayerPic {
                    small: x.gamerpic.small,
                    medium: x.gamerpic.medium,
                    large: x.gamerpic.large,
                    xlarge: x.gamerpic.xlarge,
                },
            })
    }
}

async fn index_graphiql() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/").finish()))
}

async fn index(
    data: web::Data<ActixData>,
    req: HttpRequest,
    request: GraphQLRequest,
) -> GraphQLResponse {
    let spartan_token = req
        .headers()
        .get("spartan_token")
        .map(|x| x.to_str())
        .transpose()
        .unwrap_or_default()
        .unwrap_or_default()
        .to_string();

    data.schema
        .execute(request.into_inner().data(AuthData {
            spartan_token: spartan_token.clone(),
            client: data.client.clone(),
            loader: DataLoader::new(
                HaloLoader {
                    client: data.client.clone(),
                    spartan_token: spartan_token,
                },
                actix_web::rt::spawn,
            ),
        }))
        .await
        .into()
}

struct ActixData {
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
    client: Client,
}

pub struct AuthData {
    pub spartan_token: String,
    pub client: Client,
    pub loader: DataLoader<HaloLoader>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    println!("GraphiQL IDE: http://localhost:8000");

    let client = Client::new();

    let data = web::Data::new(ActixData {
        schema: Schema::build(Query, EmptyMutation, EmptySubscription).finish(),
        client: client,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_graphiql))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
