
# Simple Key-Value Store in Rust
This is a lightweight key-value store server implemented in Rust. It provides basic CRUD operations (Create, Read, Delete) and persists data to a file. The server runs over TCP and handles HTTP-like requests.

## Features

- Operations: Supports SET (via PUT), GET, and DELETE.
- Persistence: Stores data in a file named kvstore.db.
- Concurrency: Thread-safe using Mutex and Arc for handling multiple requests.
- Dynamic: No hard limits on key/value sizes, with efficient memory management via HashMap.

## How It Works

1. The store loads existing data from kvstore.db on startup.
2. Data is kept in memory in a HashMap and synced to the file on each SET or DELETE.
3.  Deletion is implemented by writing an empty value for the key.

## Usage

Run the Server:
```bash

    cargo run

    The server starts at http://127.0.0.1:8080.
    Interact with the Server:
        Get a value: curl http://127.0.0.1:8080/kv/mykey
        Set a value: curl -X PUT -d "myvalue" http://127.0.0.1:8080/kv/mykey
        Delete a key: curl -X DELETE http://127.0.0.1:8080/kv/mykey
```
File Format

  - Each entry: ```[key_size (8 bytes)][value_size (8 bytes)][key][value].```
  - Deletion: Marked by a zero-length value.

Requirements

  - Rust (latest stable version recommended).

