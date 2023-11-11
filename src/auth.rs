use async_graphql::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct XboxTicketProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rps_ticket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_tokens: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XboxTicketRequest {
    pub relying_party: String,
    pub token_type: String,
    pub properties: XboxTicketProperties,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XboxTicketResponse {
    pub issue_instant: String,
    pub not_after: String,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpartanTokenProof {
    pub token: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpartanTokenRequest {
    pub audience: String,
    pub min_version: String,
    pub proof: Vec<SpartanTokenProof>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpartanTokenResponse {
    pub expires_utc: SpartanTokenExpiresUtc,
    pub spartan_token: String,
    pub token_duration: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpartanTokenExpiresUtc {
    #[serde(rename = "ISO8601Date")]
    pub iso8601_date: String,
}

pub async fn spartan_token(client: &Client, xsts_token: String) -> Result<SpartanTokenResponse> {
    Ok(client
        .post(env::var("SPARTAN_TOKEN_URL").expect("Missing Spartan Token Url"))
        .header("Accept", "application/json")
        .json(&SpartanTokenRequest {
            audience: String::from("urn:343:s3:services"),
            min_version: String::from("4"),
            proof: vec![SpartanTokenProof {
                token: xsts_token,
                token_type: String::from("Xbox_XSTSv3"),
            }],
        })
        .send()
        .await?
        .json::<SpartanTokenResponse>()
        .await?)
}

pub async fn xsts_token(client: &Client, user_token: String) -> Result<XboxTicketResponse> {
    Ok(client
        .post(env::var("XBOX_XSTS_URL").expect("Missing XBOX XSTS Url"))
        .header("x-xbl-contract-version", "1")
        .json(&XboxTicketRequest {
            relying_party: String::from("https://prod.xsts.halowaypoint.com/"),
            token_type: String::from("JWT"),
            properties: XboxTicketProperties {
                sandbox_id: Some(String::from("RETAIL")),
                user_tokens: Some(vec![user_token]),
                ..XboxTicketProperties::default()
            },
        })
        .send()
        .await?
        .json::<XboxTicketResponse>()
        .await?)
}

pub async fn user_token(client: &Client, access_token: String) -> Result<XboxTicketResponse> {
    Ok(client
        .post(env::var("XBOX_AUTH_URL").expect("Missing XBOX Auth Url"))
        .header("x-xbl-contract-version", "1")
        .json(&XboxTicketRequest {
            relying_party: String::from("http://auth.xboxlive.com"),
            token_type: String::from("JWT"),
            properties: XboxTicketProperties {
                auth_method: Some(String::from("RPS")),
                site_name: Some(String::from("user.auth.xboxlive.com")),
                rps_ticket: Some(format!("d={access_token}")),
                ..XboxTicketProperties::default()
            },
        })
        .send()
        .await?
        .json::<XboxTicketResponse>()
        .await?)
}

pub async fn auth_token(
    client: &Client,
    code: Option<String>,
    refresh_token: Option<String>,
) -> Result<AuthTokenResponse> {
    Ok(client
        .post(env::var("AUTH_TOKEN_URL").expect("Missing Auth Token Url"))
        .form(&[
            (
                "grant_type",
                if refresh_token.is_some() {
                    "refresh_token"
                } else {
                    "authorization_code"
                },
            ),
            (
                "code",
                refresh_token.unwrap_or(code.unwrap_or_default()).as_str(),
            ),
            (
                "client_id",
                env::var("AUTH_CLIENT_ID")
                    .expect("Missing Auth Client Url")
                    .as_str(),
            ),
            (
                "client_secret",
                env::var("AUTH_CLIENT_SECRET")
                    .expect("Missing Auth Client Secret")
                    .as_str(),
            ),
            ("approval_prompt", "auto"),
            ("scope", "Xboxlive.signin Xboxlive.offline_access"),
            (
                "redirect_uri",
                env::var("AUTH_REDIRECT_URI")
                    .expect("Missing Auth Redirect Uri")
                    .as_str(),
            ),
        ])
        .send()
        .await?
        .json::<AuthTokenResponse>()
        .await?)
}

pub fn redirect_url() -> String {
    format!(
        "{}?{}",
        env::var("AUTH_BASE_URL").expect("Missing Auth Base Url"),
        querystring::stringify(vec![
            (
                "client_id",
                env::var("AUTH_CLIENT_ID")
                    .expect("Missing Auth Client Url")
                    .as_str(),
            ),
            ("response_type", "code"),
            ("approval_prompt", "auto"),
            ("scope", "Xboxlive.signin Xboxlive.offline_access"),
            (
                "redirect_uri",
                env::var("AUTH_REDIRECT_URI")
                    .expect("Missing Auth Redirect Uri")
                    .as_str()
            ),
        ])
    )
}
