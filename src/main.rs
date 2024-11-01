mod crypto;
mod db;
mod error;
mod handlers;
mod models;

use actix_web::{web, App, HttpServer};
use crypto::CryptoContext;
use dotenv::dotenv;
use handlers::{get_stats, health_check, submit_age};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize the database connection pool
    let db_pool = db::init_pool().await.expect("Failed to create pool");

    // Initialize encryption context
    let crypto_context = CryptoContext::new().expect("Failed to initialize crypto context");

    println!("Server starting at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(crypto_context.clone()))
            .service(health_check)
            .service(submit_age)
            .service(get_stats)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
