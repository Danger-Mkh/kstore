
# Security
This document outlines security considerations for the Simple Key-Value Store server implemented in Rust.

## Security Features

- Thread Safety: Uses Mutex and Arc to prevent data races in concurrent access.
- File-Based Persistence: Data is stored in kvstore.db with no default encryption (see below).

## Known Security Considerations

1. No Authentication:
  - The server does not implement user authentication or authorization.
  - Anyone with network access to 127.0.0.1:8080 can read, write, or delete data.
2. Unencrypted Storage:
  - Data in kvstore.db is stored in plaintext.
  - Sensitive data could be exposed if the file is accessed by an unauthorized party.
3. No Input Validation:
  - Keys and values are not sanitized, which could allow malformed requests to cause unexpected behavior.
  - Large inputs could potentially exhaust memory or disk space (no size limits enforced).
4. Localhost Only:
  - By default, the server binds to 127.0.0.1, limiting access to the local machine.
  - Changing the bind address (e.g., to 0.0.0.0) exposes it to the network, increasing risk.

## Recommendations

- Add Authentication: Implement basic auth (e.g., API keys) for production use.
- Enable Encryption: Use a library like rust-crypto to encrypt data before writing to kvstore.db.
- Limit Input Size: Add checks to restrict key/value sizes (e.g., 1MB max).
- Run Behind a Proxy: Use a reverse proxy (e.g., Nginx) with TLS for secure network access.
- File Permissions: Ensure kvstore.db has strict permissions (e.g., chmod 600).

## Reporting Issues
If you discover a security vulnerability, please report it privately by contacting the maintainer directly (add your contact info here, e.g., email or Telegram). We’ll address it promptly and credit you if desired.
## Dependencies
This project relies on the Rust standard library only. Ensure you keep your Rust toolchain updated (rustup update) to benefit from security patches.
