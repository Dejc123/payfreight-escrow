use serde::{Deserialize, Serialize};
use reqwest::Client;
use actix_web::{web, HttpResponse, Error};

#[derive(Deserialize)]
pub struct LinkedInAuthRequest {
    pub code: String,
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

    // 1. Izmenjava 'code' za 'access_token' pri LinkedInu
    let token_res = client
        .post("https://www.linkedin.com/oauth/v2/accessToken")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &payload.code),
            ("client_id", "770xgs9ozmyrwn"),
            ("client_secret", "WPL_AP1.KBbEsl8dFglyJaJC.okOUOA=="), // Vstavi svoj skriti Client Secret!
            ("redirect_uri", "https://www.payfreight.io/cargotrans/register_cargonet.html"),
        ])
        .send()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Token request failed"))?;

    let token_data: LinkedInTokenResponse = token_res
        .json()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to parse token"))?;

    // 2. Pridobivanje resničnega profila prek OpenID userinfo endpointa
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
