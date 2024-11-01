use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AgeSubmission {
    pub encrypted_age: String, // Base64 encoded encrypted age
    pub user_id: String,
}

#[derive(Serialize)]
pub struct AgeStats {
    pub total_users: i64,
    pub users_under_25_encrypted: String, // Base64 encoded encrypted count
    pub users_under_35_encrypted: String, // Base64 encoded encrypted count
}
