use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use tfhe::{
    integer::{gen_keys_radix, RadixCiphertext, RadixClientKey, ServerKey},
    shortint::parameters::PARAM_MESSAGE_2_CARRY_2,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to execute (submit, stats, or generate-keys)
    #[arg(short, long)]
    command: String,

    /// Age to submit (only needed for submit command)
    #[arg(short, long)]
    age: Option<u32>,
}

#[derive(Serialize)]
struct AgeSubmission {
    encrypted_age: String,
    user_id: String,
}

#[derive(Deserialize, Debug)]
struct AgeStats {
    total_users: i64,
    users_under_25_encrypted: String,
    users_under_35_encrypted: String,
}

#[derive(Clone)]
pub struct ClientCryptoContext {
    client_key: RadixClientKey,
}

impl ClientCryptoContext {
    pub fn encrypt(&self, age: u32) -> Result<RadixCiphertext, Box<dyn std::error::Error>> {
        if age > 255 {
            return Err("Age must be less than 256".into());
        }

        println!("Encrypting age: {}", age);
        let encrypted = self.client_key.encrypt(age as u64);
        Ok(encrypted)
    }

    pub fn decrypt(&self, ciphertext: &RadixCiphertext) -> Result<u8, Box<dyn std::error::Error>> {
        let value: u64 = self.client_key.decrypt(ciphertext);
        println!("Decrypted value: {}", value);
        Ok(value as u8)
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load keys from file
        let file = File::open("server_keys.bincode")?;
        let reader = BufReader::new(file);
        let (client_key, _): (RadixClientKey, ServerKey) = bincode::deserialize_from(reader)?;

        Ok(Self { client_key })
    }

    pub fn generate_and_save_keys() -> Result<Self, Box<dyn std::error::Error>> {
        let (client_key, server_key) = gen_keys_radix(PARAM_MESSAGE_2_CARRY_2, 8);

        // Save both keys to file
        let file = File::create("server_keys.bincode")?;
        bincode::serialize_into(file, &(client_key.clone(), server_key))?;

        Ok(Self { client_key })
    }
}

struct Client {
    crypto_context: ClientCryptoContext,
    http_client: reqwest::Client,
    base_url: String,
}

impl Client {
    fn new() -> Result<Self, Box<dyn Error>> {
        let crypto_context = ClientCryptoContext::new()?;
        let http_client = reqwest::Client::new();
        Ok(Self {
            crypto_context,
            http_client,
            base_url: "http://localhost:8080".to_string(),
        })
    }

    async fn submit_age(&self, age: u32) -> Result<(), Box<dyn Error>> {
        // Encrypt the age
        let ciphertext = self.crypto_context.encrypt(age)?;

        // Convert to base64
        let serialized = bincode::serialize(&ciphertext)?;
        let encrypted_base64 = BASE64.encode(serialized);

        // Create submission
        let submission = AgeSubmission {
            encrypted_age: encrypted_base64,
            user_id: uuid::Uuid::new_v4().to_string(),
        };

        // Send to server
        let response = self
            .http_client
            .post(format!("{}/submit-age", self.base_url))
            .json(&submission)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to submit age: {}", response.status()).into());
        }

        println!("Successfully submitted age!");
        Ok(())
    }

    async fn get_stats(&self) -> Result<(), Box<dyn Error>> {
        // Get encrypted stats from server
        let response = self
            .http_client
            .get(format!("{}/stats", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get stats: {}", response.status()).into());
        }

        let stats: AgeStats = response.json().await?;
        println!("Total users: {}", stats.total_users);

        // Decrypt under 25 count
        if let Ok(ct_25) = self.decode_ciphertext(&stats.users_under_25_encrypted) {
            let count_under_25 = self.crypto_context.decrypt(&ct_25)?;
            println!("Users under 25: {}", count_under_25);
        }

        // Decrypt under 35 count
        if let Ok(ct_35) = self.decode_ciphertext(&stats.users_under_35_encrypted) {
            let count_under_35 = self.crypto_context.decrypt(&ct_35)?;
            println!("Users under 35: {}", count_under_35);
        }

        Ok(())
    }

    fn decode_ciphertext(&self, encoded: &str) -> Result<RadixCiphertext, Box<dyn Error>> {
        let bytes = BASE64.decode(encoded)?;
        let ciphertext: RadixCiphertext = bincode::deserialize(&bytes)?;
        Ok(ciphertext)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command.as_str() {
        "generate-keys" => {
            println!("Generating new server keys...");
            ClientCryptoContext::generate_and_save_keys()?;
            println!("Keys generated and saved to server_keys.bincode");
            return Ok(());
        }
        _ => {
            let client = Client::new()?;

            match args.command.as_str() {
                "submit" => {
                    let age = args.age.ok_or("Age is required for submit command")?;
                    if age > 120 {
                        return Err("Age must be less than 120".into());
                    }
                    client.submit_age(age).await?;
                }
                "stats" => {
                    client.get_stats().await?;
                }
                _ => {
                    return Err("Invalid command. Use 'generate-keys', 'submit' or 'stats'".into());
                }
            }
        }
    }

    Ok(())
}
