use crate::{
    crypto::CryptoContext,
    db,
    models::{AgeStats, AgeSubmission},
};
use actix_web::{get, post, web, HttpResponse, Responder};
use futures::StreamExt;

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Healthy")
}

#[post("/submit-age")]
pub async fn submit_age(
    submission: web::Json<AgeSubmission>,
    db_pool: web::Data<sqlx::PgPool>,
    crypto_context: web::Data<CryptoContext>,
) -> impl Responder {
    if let Err(e) = db::store_age(&db_pool, &submission).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    HttpResponse::Ok().finish()
}

#[get("/stats")]
pub async fn get_stats(
    db_pool: web::Data<sqlx::PgPool>,
    crypto_context: web::Data<CryptoContext>,
) -> impl Responder {
    // Get total number of users
    let total_users = match db::get_total_users(&db_pool).await {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
        }
    };

    // Get all encrypted ages
    let encrypted_ages = match db::get_all_encrypted_ages(&db_pool).await {
        Ok(ages) => ages,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
        }
    };

    // Convert all base64 strings to Ciphertexts
    let mut ciphertexts = Vec::new();
    for encrypted_age in encrypted_ages {
        match crypto_context.decode_encrypted_age(&encrypted_age) {
            Ok(ct) => ciphertexts.push(ct),
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Error decoding encrypted age: {}", e));
            }
        }
    }

    // Compute homomorphic comparisons for all ages
    let mut under_25_results = Vec::new();
    let mut under_35_results = Vec::new();

    for ct in &ciphertexts {
        // Compute age < 25
        if let Ok(result_25) = crypto_context.homomorphic_less_than(ct, 25) {
            under_25_results.push(result_25);
        }

        // Compute age < 35
        if let Ok(result_35) = crypto_context.homomorphic_less_than(ct, 35) {
            under_35_results.push(result_35);
        }
    }

    // Sum up the results homomorphically
    let users_under_25 = match crypto_context.homomorphic_sum(&under_25_results) {
        Ok(sum) => {
            // Encode the encrypted sum for transmission
            crypto_context
                .encode_encrypted_result(&sum)
                .unwrap_or_else(|_| "0".to_string())
        }
        Err(_) => "0".to_string(),
    };

    let users_under_35 = match crypto_context.homomorphic_sum(&under_35_results) {
        Ok(sum) => crypto_context
            .encode_encrypted_result(&sum)
            .unwrap_or_else(|_| "0".to_string()),
        Err(_) => "0".to_string(),
    };

    let stats = AgeStats {
        total_users,
        users_under_25_encrypted: users_under_25,
        users_under_35_encrypted: users_under_35,
    };

    HttpResponse::Ok().json(stats)
}
