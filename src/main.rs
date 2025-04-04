use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, Seek, SeekFrom};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

struct KvStore {
    data: Mutex<HashMap<String, String>>,
    file: Mutex<File>,
}

impl KvStore {
    fn new() -> Self {
        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("kvstore.db")
        .unwrap();

        let mut data = HashMap::new();
        let mut reader = BufReader::new(&file);
        let mut buffer = Vec::new();

        if reader.read_to_end(&mut buffer).is_ok() {
            let mut pos = 0;
            while pos < buffer.len() {
                if buffer.len() - pos < 16 {
                    break;
                }

                let key_size = u64::from_le_bytes(buffer[pos..pos + 8].try_into().unwrap()) as usize;
                pos += 8;
                let value_size = u64::from_le_bytes(buffer[pos..pos + 8].try_into().unwrap()) as usize;
                pos += 8;

                if pos + key_size + value_size > buffer.len() {
                    break;
                }

                let key = String::from_utf8_lossy(&buffer[pos..pos + key_size]).to_string();
                pos += key_size;
                let value = String::from_utf8_lossy(&buffer[pos..pos + value_size]).to_string();
                pos += value_size;

                if !value.is_empty() {
                    data.insert(key, value);
                } else {
                    data.remove(&key);
                }
            }
        }

        file.seek(SeekFrom::End(0)).unwrap();
        Self {
            data: Mutex::new(data),
            file: Mutex::new(file),
        }
    }

    fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().unwrap();
        let mut file = self.file.lock().unwrap();

        data.insert(key.clone(), value.clone());
        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();

        file.write_all(&(key_bytes.len() as u64).to_le_bytes()).unwrap();
        file.write_all(&(value_bytes.len() as u64).to_le_bytes()).unwrap();
        file.write_all(key_bytes).unwrap();
        file.write_all(value_bytes).unwrap();
        file.flush().unwrap();
    }

    fn get(&self, key: &str) -> Option<String> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    fn delete(&self, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        let mut file = self.file.lock().unwrap();

        if data.remove(key).is_some() {
            let key_bytes = key.as_bytes();
            file.write_all(&(key_bytes.len() as u64).to_le_bytes()).unwrap();
            file.write_all(&0u64.to_le_bytes()).unwrap(); // value_size = 0
            file.write_all(key_bytes).unwrap();
            file.flush().unwrap();
            true
        } else {
            false
        }
    }
}

fn handle_client(mut stream: TcpStream, store: Arc<KvStore>) {
    let mut buffer = [0; 1024];
    if stream.read(&mut buffer).is_err() {
        return;
    }

    let request = String::from_utf8_lossy(&buffer);
    let parts: Vec<&str> = request.lines().next().unwrap_or("").split_whitespace().collect();

    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];
    let key = path.strip_prefix("/kv/").unwrap_or("");

    let response = match method {
        "GET" => {
            if let Some(value) = store.get(key) {
                format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", value.len(), value)
            } else {
                "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n".to_string()
            }
        }
        "PUT" => {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or("").trim();
            store.set(key.to_string(), body.to_string());
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK".to_string()
        }
        "DELETE" => {
            if store.delete(key) {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK".to_string()
            } else {
                "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n".to_string()
            }
        }
        _ => "HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 0\r\n\r\n".to_string(),
    };

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let store = Arc::new(KvStore::new());

    println!("Server running at http://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = Arc::clone(&store);
                thread::spawn(|| handle_client(stream, store));
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
