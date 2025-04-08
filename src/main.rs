use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

use actix_web::middleware::{Compress, Logger};
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use env_logger::Env;
use regex::Regex;

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

                let key_size =
                    u64::from_le_bytes(buffer[pos..pos + 8].try_into().unwrap()) as usize;
                pos += 8;
                let value_size =
                    u64::from_le_bytes(buffer[pos..pos + 8].try_into().unwrap()) as usize;
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
        file.write_all(&(key_bytes.len() as u64).to_le_bytes())
            .unwrap();
        file.write_all(&(value_bytes.len() as u64).to_le_bytes())
            .unwrap();
        file.write_all(key_bytes).unwrap();
        file.write_all(value_bytes).unwrap();
        file.flush().unwrap();
    }

    fn get(&self, key: &str) -> Option<String> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    fn compact(&self) {
        let data = self.data.lock().unwrap();
        let mut file = self.file.lock().unwrap();

        file.set_len(0).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        for (key, value) in data.iter() {
            let key_bytes = key.as_bytes();
            let value_bytes = value.as_bytes();
            file.write_all(&(key_bytes.len() as u64).to_le_bytes())
                .unwrap();
            file.write_all(&(value_bytes.len() as u64).to_le_bytes())
                .unwrap();
            file.write_all(key_bytes).unwrap();
            file.write_all(value_bytes).unwrap();
        }
        file.flush().unwrap();
    }

    fn delete(&self, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        if data.remove(key).is_some() {
            drop(data);
            self.compact();
            true
        } else {
            false
        }
    }

    fn find_values_by_regex(&self, pattern: &str) -> Result<Vec<String>, regex::Error> {
        let re = Regex::new(pattern)?;
        let data = self.data.lock().unwrap();
        let values: Vec<String> = data
            .iter()
            .filter(|(key, _)| re.is_match(key))
            .map(|(_, value)| value.clone())
            .collect();
        Ok(values)
    }
}

async fn get_all_keys(store: web::Data<KvStore>) -> impl Responder {
    let data = store.data.lock().unwrap();
    let keys: Vec<String> = data.keys().cloned().collect();
    if keys.is_empty() {
        HttpResponse::NotFound().json(vec![] as Vec<String>)
    } else {
        HttpResponse::Ok().json(keys)
    }
}
async fn get_key(store: web::Data<KvStore>, path: web::Path<String>) -> impl Responder {
    let key = path.into_inner();
    match store.get(&key) {
        Some(value) => HttpResponse::Ok().body(value),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn put_key(
    store: web::Data<KvStore>,
    path: web::Path<String>,
    body: String,
) -> impl Responder {
    let key = path.into_inner();
    match store.get(&key) {
        Some(_val) => HttpResponse::BadRequest().body("key already exist"),
        None => {
            store.set(key, body);
            HttpResponse::Ok().body("OK")
        }
    }
}

async fn update_key(
    store: web::Data<KvStore>,
    path: web::Path<String>,
    body: String,
) -> impl Responder {
    let key = path.into_inner();
    match store.get(&key) {
        Some(_val) => {
            store.set(key, body);
            HttpResponse::Ok().body("OK")
        }
        None => HttpResponse::BadRequest().body("key does not exist"),
    }
}

async fn delete_key(store: web::Data<KvStore>, path: web::Path<String>) -> impl Responder {
    let key = path.into_inner();
    if store.delete(&key) {
        HttpResponse::Ok().body("OK")
    } else {
        HttpResponse::NotFound().finish()
    }
}

async fn get_values_by_regex(store: web::Data<KvStore>, path: web::Path<String>) -> impl Responder {
    let pattern = path.into_inner();
    match store.find_values_by_regex(&pattern) {
        Ok(values) => {
            if values.is_empty() {
                HttpResponse::NotFound().body("No values matched the pattern")
            } else {
                let response = values.join("\n");
                HttpResponse::Ok().body(response)
            }
        }
        Err(e) => HttpResponse::BadRequest().body(format!("Invalid regex pattern: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store = web::Data::new(KvStore::new());
    println!("Server running at http://127.0.0.1:8080");
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .route("/kv/", web::get().to(get_all_keys))
            .route("/kv/{key}", web::get().to(get_key))
            .route("/kv/{key}", web::post().to(put_key))
            .route("/kv/{key}", web::put().to(update_key))
            .route("/kv/{key}", web::delete().to(delete_key))
            .route("/kv/r/{regex}", web::get().to(get_values_by_regex))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
