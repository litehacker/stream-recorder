use axum::{
    extract::State,
    Json,
};
use uuid::Uuid;
use crate::models::User;
use crate::AppState;

pub async fn generate_credentials(
    State(state): State<AppState>,
) -> Json<User> {
    let user = User {
        id: Uuid::new_v4(),
        api_key: generate_api_key(),
        quota_limit: 1_000_000_000, // 1GB default
        quota_used: 0,
    };

    // Save to database
    sqlx::query!(
        "INSERT INTO users (id, api_key, quota_limit, quota_used) VALUES ($1, $2, $3, $4)",
        user.id,
        user.api_key,
        user.quota_limit,
        user.quota_used
    )
    .execute(&state.db)
    .await
    .unwrap();

    Json(user)
}

fn generate_api_key() -> String {
    Uuid::new_v4().to_string().replace("-", "")
} 