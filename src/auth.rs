use actix_web::{web, Error, HttpResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LinkedInAuthRequest {
    pub code: String,
    pub redirect_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub email: String,
    pub name: String,
    pub given_name: String,
}

#[derive(Deserialize)]
struct LinkedInTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct LinkedInUserInfo {
    email: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
}

pub async fn handle_linkedin_auth(
    payload: web::Json<LinkedInAuthRequest>,
) -> Result<HttpResponse, Error> {
    let client = Client::new();

    // 1. Resolve redirect URI (fallback to default if not provided by frontend)
    let redirect_uri = payload
        .redirect_uri
        .clone()
        .unwrap_or_else(|| "https://www.payfreight.io/cargotrans/register_cargonet.html".to_string());

    // 2. Retrieve client secret from environment variable or fallback
    let client_secret = std::env::var("LINKEDIN_CLIENT_SECRET")
        .unwrap_or_else(|_| "WPL_AP1.KBbEsl8dFglyJaJC.okOUOA==".to_string());

    // 3. Exchange 'code' for 'access_token' with LinkedIn OAuth server
    let token_res = client
        .post("https://www.linkedin.com/oauth/v2/accessToken")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &payload.code),
            ("client_id", "770xgs9ozmyrwn"),
            ("client_secret", &client_secret),
            ("redirect_uri", &redirect_uri),
        ])
        .send()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Token request failed"))?;

    let token_data: LinkedInTokenResponse = token_res
        .json()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to parse token"))?;

    // 4. Fetch real user profile data via OpenID userinfo endpoint
    let user_info_res = client
        .get("https://api.linkedin.com/v2/userinfo")
        .bearer_auth(token_data.access_token)
        .send()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Userinfo request failed"))?;

    let user_info: LinkedInUserInfo = user_info_res
        .json()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to parse userinfo"))?;

    let response = UserProfileResponse {
        email: user_info.email.unwrap_or_default(),
        name: user_info.name.unwrap_or_default(),
        given_name: user_info.given_name.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(response))
}
