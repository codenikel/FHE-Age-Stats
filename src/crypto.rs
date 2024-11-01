use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode::{deserialize, serialize};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tfhe::integer::{BooleanBlock, RadixCiphertext, RadixClientKey, ServerKey};

#[derive(Clone)]
pub struct CryptoContext {
    server_key: Arc<ServerKey>,
}

impl CryptoContext {
    pub fn homomorphic_less_than(
        &self,
        encrypted_age: &RadixCiphertext,
        threshold: u32,
    ) -> Result<RadixCiphertext, Box<dyn std::error::Error>> {
        // Convert threshold to u64 and create comparison
        let comparison: BooleanBlock = self
            .server_key
            .scalar_lt_parallelized(encrypted_age, threshold as u64);

        // Create trivial ciphertexts for 0 and 1
        let one: RadixCiphertext = self.server_key.create_trivial_radix(1u64, 8);
        let zero: RadixCiphertext = self.server_key.create_trivial_radix(0u64, 8);

        // Convert boolean block to radix ciphertext
        let comp_radix: RadixCiphertext = comparison.into_radix(8, &self.server_key);

        // Use scalar multiplication to select between 0 and 1
        let final_result = self.server_key.scalar_mul_parallelized(&comp_radix, 1u64);

        Ok(final_result)
    }

    pub fn decode_encrypted_age(
        &self,
        encoded: &str,
    ) -> Result<RadixCiphertext, Box<dyn std::error::Error>> {
        let bytes = BASE64.decode(encoded)?;
        let ciphertext: RadixCiphertext = deserialize(&bytes)?;
        Ok(ciphertext)
    }

    pub fn encode_encrypted_result(
        &self,
        ciphertext: &RadixCiphertext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let serialized = serialize(ciphertext)?;
        Ok(BASE64.encode(serialized))
    }

    pub fn homomorphic_sum(
        &self,
        ciphertexts: &[RadixCiphertext],
    ) -> Result<RadixCiphertext, Box<dyn std::error::Error>> {
        if ciphertexts.is_empty() {
            return Err("Empty ciphertext array".into());
        }

        let mut sum = ciphertexts[0].clone();
        for ct in &ciphertexts[1..] {
            sum = self.server_key.add_parallelized(&sum, ct);
        }
        Ok(sum)
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load keys from file
        let file = File::open("server_keys.bincode")?;
        let reader = BufReader::new(file);
        let (_client_key, server_key): (RadixClientKey, ServerKey) =
            bincode::deserialize_from(reader)?;

        Ok(CryptoContext {
            server_key: Arc::new(server_key),
        })
    }
}
