# Private Age Statistics

A privacy-preserving application that allows users to submit their ages and compute aggregate statistics without revealing individual ages. The application uses homomorphic encryption to perform computations on encrypted data.

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Docker (optional, for running PostgreSQL)

## Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd private-age-stats
   ```

2. Set up the database:
   ```bash
   docker run --name age-stats-db -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres
   ```

3. Create the database:
   ```bash
   psql -U postgres -h localhost -c "CREATE DATABASE age_stats;"
   ```

4. Create a `.env` file in the project root:
   ```
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/age_stats
   ```

5. Run the database migrations:
   ```bash
   psql -U postgres -h localhost -d age_stats -f migrations/001_initial_schema.sql
   ```

## Running the Application

### Generate keys

```bash
cargo run --bin client -- -c generate-keys
```

### Server

1. Start the server:
   ```bash
   cargo run --bin server
   ```

The server will start at `http://localhost:8080`

### Client

The client provides two commands:

1. Submit an age:
   ```bash
   cargo run --bin client -- -c submit -a 25
   ```

2. Get statistics:
   ```bash
   cargo run --bin client -- -c stats
   ```

## API Endpoints

- `GET /health` - Health check endpoint
- `POST /submit-age` - Submit an encrypted age
- `GET /stats` - Get encrypted aggregate statistics

## How it Works

1. When a user submits their age through the client:
   - The age is encrypted locally using homomorphic encryption
   - The encrypted data is sent to the server
   - The server stores the encrypted data without being able to decrypt it

2. When requesting statistics:
   - The server performs computations on the encrypted data
   - Results are returned in encrypted form
   - The client decrypts the results locally

This ensures that:
- Individual ages are never revealed to the server
- The server can still compute useful statistics
- Only authorized clients can decrypt the results

## Security Considerations

- All data is encrypted using homomorphic encryption
- The server never sees plaintext ages
- Each user gets a unique identifier
- The server can compute statistics without decrypting data

## Development

To run tests:
```bash
cargo test
```

To format code:
```bash
cargo fmt
```

To check for common issues:
```bash
cargo clippy
```

## Project Structure
```
.
├── Cargo.toml
├── README.md
├── migrations/
│   └── 001_initial_schema.sql
└── src/
    ├── main.rs           # Server entry point
    ├── crypto.rs         # Encryption logic
    ├── models.rs         # Data structures
    ├── handlers.rs       # API endpoints
    ├── db.rs            # Database operations
    └── bin/
        └── client.rs     # CLI client
```

## Troubleshooting

1. If you get database connection errors:
   - Make sure PostgreSQL is running
   - Check that the DATABASE_URL in .env is correct
   - Verify that the database exists

2. If you get "permission denied" errors:
   - Check PostgreSQL user permissions
   - Make sure you're using the correct password

3. If the client can't connect to the server:
   - Verify the server is running
   - Check that you're using the correct port (8080)

## Production Considerations

For production deployment, consider:
- Using HTTPS
- Implementing user authentication
- Adding rate limiting
- Setting up proper logging
- Implementing key rotation
- Adding monitoring and alerting
- Using connection pooling for the database
- Setting up backups for the database

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

[MIT License](LICENSE)