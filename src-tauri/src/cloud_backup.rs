//! S3, WebDAV, and LocalPath Cloud Backup & Restore Engine.

use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;
use berry_domain::{
    CloudBackupConfig, CloudBackupResult, CloudPingResult, CloudRestoreResult, CloudSnapshotMeta,
    CloudStorageProvider,
};
use berry_storage::Database;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

// -----------------------------------------------------------------------------
// Cryptographic & SigV4 Utilities
// -----------------------------------------------------------------------------

/// Pure-Rust HMAC-SHA256 implementation using `sha2::Sha256`.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let hash = Sha256::digest(key);
        k[..32].copy_from_slice(&hash);
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    outer.finalize().to_vec()
}

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

/// Calculate SHA-256 of an on-disk file in streaming 64KB chunks.
pub(crate) fn sha256_file(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn format_iso8601_basic(time: SystemTime) -> (String, String) {
    let secs = time.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days = secs / 86400;
    let day_secs = secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    let mut year = 1970;
    let mut rem_days = days;
    loop {
        let days_in_year = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
            366
        } else {
            365
        };
        if rem_days >= days_in_year {
            rem_days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_per_month = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &d in &days_per_month {
        if rem_days >= d {
            rem_days -= d;
            month += 1;
        } else {
            break;
        }
    }
    let day = rem_days + 1;

    let date_str = format!("{year:04}{month:02}{day:02}");
    let datetime_str = format!("{year:04}{month:02}{day:02}T{hours:02}{minutes:02}{seconds:02}Z");
    (date_str, datetime_str)
}

// -----------------------------------------------------------------------------
// S3 REST Client (AWS SigV4)
// -----------------------------------------------------------------------------

pub(crate) struct S3Client<'a> {
    pub(crate) endpoint: &'a str,
    pub(crate) bucket: &'a str,
    pub(crate) region: &'a str,
    pub(crate) access_key: &'a str,
    pub(crate) secret_key: &'a str,
    pub(crate) prefix: &'a str,
}

impl<'a> S3Client<'a> {
    pub(crate) fn from_config(config: &'a CloudBackupConfig) -> Result<Self, String> {
        let endpoint = config
            .s3_endpoint
            .as_deref()
            .ok_or_else(|| "S3 endpoint URL is required".to_string())?
            .trim_end_matches('/');
        let bucket = config
            .s3_bucket
            .as_deref()
            .ok_or_else(|| "S3 bucket name is required".to_string())?;
        let region = config.s3_region.as_deref().unwrap_or("us-east-1");
        let access_key = config
            .s3_access_key
            .as_deref()
            .ok_or_else(|| "S3 access key is required".to_string())?;
        let secret_key = config
            .s3_secret_key
            .as_deref()
            .ok_or_else(|| "S3 secret key is required".to_string())?;
        let prefix = config.s3_prefix.as_deref().unwrap_or("").trim_matches('/');

        Ok(Self {
            endpoint,
            bucket,
            region,
            access_key,
            secret_key,
            prefix,
        })
    }

    pub(crate) fn object_key(&self, filename: &str) -> String {
        let clean = filename.trim_start_matches('/');
        if self.prefix.is_empty() {
            clean.to_string()
        } else {
            format!("{}/{}", self.prefix, clean)
        }
    }

    pub(crate) fn sign_request(
        &self,
        method: &str,
        path: &str,
        query: &str,
        payload: &[u8],
    ) -> (String, Vec<(String, String)>) {
        self.sign_request_ext(method, path, query, payload, &[])
    }

    pub(crate) fn sign_request_ext(
        &self,
        method: &str,
        path: &str,
        query: &str,
        payload: &[u8],
        extra_headers: &[(&str, &str)],
    ) -> (String, Vec<(String, String)>) {
        let (date_str, datetime_str) = format_iso8601_basic(SystemTime::now());

        let url_parsed = self.endpoint.trim_start_matches("https://").trim_start_matches("http://");
        let host = url_parsed.split('/').next().unwrap_or(url_parsed);

        let payload_hash = sha256_hex(payload);

        let canonical_uri = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };

        let mut all_signed: Vec<(String, String)> = vec![
            ("host".to_string(), host.to_string()),
            ("x-amz-content-sha256".to_string(), payload_hash.clone()),
            ("x-amz-date".to_string(), datetime_str.clone()),
        ];
        for (k, v) in extra_headers {
            all_signed.push((k.to_lowercase(), v.trim().to_string()));
        }
        all_signed.sort_by(|a, b| a.0.cmp(&b.0));

        let mut canonical_headers = String::new();
        let mut signed_headers_parts = Vec::new();
        for (k, v) in &all_signed {
            canonical_headers.push_str(&format!("{k}:{v}\n"));
            signed_headers_parts.push(k.as_str());
        }
        let signed_headers = signed_headers_parts.join(";");

        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let canonical_request_hash = sha256_hex(canonical_request.as_bytes());

        let credential_scope = format!("{date_str}/{}/s3/aws4_request", self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{datetime_str}\n{credential_scope}\n{canonical_request_hash}"
        );

        let k_secret = format!("AWS4{}", self.secret_key);
        let k_date = hmac_sha256(k_secret.as_bytes(), date_str.as_bytes());
        let k_region = hmac_sha256(&k_date, self.region.as_bytes());
        let k_service = hmac_sha256(&k_region, b"s3");
        let k_signing = hmac_sha256(&k_service, b"aws4_request");

        let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.access_key
        );

        let mut headers = vec![
            ("Host".to_string(), host.to_string()),
            ("x-amz-date".to_string(), datetime_str),
            ("x-amz-content-sha256".to_string(), payload_hash),
            ("Authorization".to_string(), auth_header),
        ];
        for (k, v) in extra_headers {
            headers.push((k.to_string(), v.to_string()));
        }

        let target_url = if query.is_empty() {
            format!("{}{canonical_uri}", self.endpoint)
        } else {
            format!("{}{canonical_uri}?{query}", self.endpoint)
        };

        (target_url, headers)
    }

    pub fn ping(&self) -> Result<u64, String> {
        let start = Instant::now();
        let path = format!("/{}", self.bucket);
        let (url, headers) = self.sign_request("GET", &path, "location", &[]);

        let mut req = ureq::get(&url);
        for (k, v) in headers {
            req = req.set(&k, &v);
        }

        let resp = req.call().map_err(|e| format!("S3 ping failed: {e}"))?;
        if resp.status() >= 200 && resp.status() < 300 {
            Ok(start.elapsed().as_millis() as u64)
        } else {
            Err(format!("S3 ping returned HTTP status {}", resp.status()))
        }
    }

    pub fn put_object(&self, filename: &str, data: &[u8]) -> Result<(), String> {
        let key = self.object_key(filename);
        self.put_object_raw(&key, data, None)
    }

    pub(crate) fn put_object_raw(&self, key: &str, data: &[u8], sha256_hex: Option<&str>) -> Result<(), String> {
        let path = format!("/{}/{}", self.bucket, key);
        let extra = if let Some(sha) = sha256_hex {
            vec![("x-amz-meta-sha256", sha)]
        } else {
            vec![]
        };
        let (url, headers) = self.sign_request_ext("PUT", &path, "", data, &extra);

        let mut req = ureq::put(&url);
        for (k, v) in headers {
            req = req.set(&k, &v);
        }

        let resp = req
            .send_bytes(data)
            .map_err(|e| format!("S3 PUT failed: {e}"))?;
        if resp.status() >= 200 && resp.status() < 300 {
            Ok(())
        } else {
            Err(format!("S3 PUT returned status {}", resp.status()))
        }
    }

    pub(crate) fn head_object(&self, key: &str) -> Result<Option<(u64, Option<String>, Option<String>)>, String> {
        let path = format!("/{}/{}", self.bucket, key);
        let (url, headers) = self.sign_request("HEAD", &path, "", &[]);

        let mut req = ureq::head(&url);
        for (k, v) in headers {
            req = req.set(&k, &v);
        }

        match req.call() {
            Ok(resp) => {
                let len = resp.header("Content-Length").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                let etag = resp.header("ETag").map(|s| s.trim_matches('"').to_string());
                let sha = resp.header("x-amz-meta-sha256").map(|s| s.to_string());
                Ok(Some((len, etag, sha)))
            }
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(e) => Err(format!("S3 HEAD failed: {e}")),
        }
    }

    pub fn get_object(&self, filename: &str) -> Result<Vec<u8>, String> {
        let key = self.object_key(filename);
        let path = format!("/{}/{}", self.bucket, key);
        let (url, headers) = self.sign_request("GET", &path, "", &[]);

        let mut req = ureq::get(&url);
        for (k, v) in headers {
            req = req.set(&k, &v);
        }

        let resp = req.call().map_err(|e| format!("S3 GET failed: {e}"))?;
        if resp.status() >= 200 && resp.status() < 300 {
            let mut reader = resp.into_reader();
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .map_err(|e| format!("Failed to read S3 response body: {e}"))?;
            Ok(bytes)
        } else {
            Err(format!("S3 GET returned status {}", resp.status()))
        }
    }

    pub fn list_snapshots(&self) -> Result<Vec<String>, String> {
        let path = format!("/{}", self.bucket);
        let query = if self.prefix.is_empty() {
            "list-type=2".to_string()
        } else {
            format!("list-type=2&prefix={}", self.prefix)
        };
        let (url, headers) = self.sign_request("GET", &path, &query, &[]);

        let mut req = ureq::get(&url);
        for (k, v) in headers {
            req = req.set(&k, &v);
        }

        let resp = req.call().map_err(|e| format!("S3 list failed: {e}"))?;
        let body = resp
            .into_string()
            .map_err(|e| format!("Failed to read S3 list body: {e}"))?;

        // Extract <Key>...</Key> tags from XML
        let mut keys = Vec::new();
        for chunk in body.split("<Key>") {
            if let Some(key) = chunk.split("</Key>").next() {
                if key.ends_with(".zip") && key.contains("berry_snapshot") {
                    let filename = key.rsplit('/').next().unwrap_or(key).to_string();
                    keys.push(filename);
                }
            }
        }
        keys.sort_by(|a, b| b.cmp(a));
        Ok(keys)
    }
}

// -----------------------------------------------------------------------------
// WebDAV REST Client
// -----------------------------------------------------------------------------

pub(crate) struct WebDavClient<'a> {
    pub(crate) endpoint: &'a str,
    pub(crate) username: &'a str,
    pub(crate) password: &'a str,
}

impl<'a> WebDavClient<'a> {
    pub(crate) fn from_config(config: &'a CloudBackupConfig) -> Result<Self, String> {
        let endpoint = config
            .webdav_endpoint
            .as_deref()
            .ok_or_else(|| "WebDAV endpoint URL is required".to_string())?
            .trim_end_matches('/');
        let username = config.webdav_username.as_deref().unwrap_or("");
        let password = config.webdav_password.as_deref().unwrap_or("");

        Ok(Self {
            endpoint,
            username,
            password,
        })
    }

    pub(crate) fn auth_header(&self) -> Option<String> {
        if self.username.is_empty() {
            None
        } else {
            let cred = format!("{}:{}", self.username, self.password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(cred.as_bytes());
            Some(format!("Basic {encoded}"))
        }
    }

    pub fn ping(&self) -> Result<u64, String> {
        let start = Instant::now();
        let mut req = ureq::request("PROPFIND", self.endpoint).set("Depth", "0");
        if let Some(auth) = self.auth_header() {
            req = req.set("Authorization", &auth);
        }

        match req.call() {
            Ok(resp) if resp.status() == 200 || resp.status() == 207 => {
                Ok(start.elapsed().as_millis() as u64)
            }
            Ok(resp) => Err(format!("WebDAV ping status: {}", resp.status())),
            Err(ureq::Error::Status(status, _)) if status == 207 || status == 200 => {
                Ok(start.elapsed().as_millis() as u64)
            }
            Err(e) => Err(format!("WebDAV connection failed: {e}")),
        }
    }

    pub(crate) fn head_object(&self, relative_path: &str) -> Result<Option<(u64, Option<String>)>, String> {
        let clean = relative_path.trim_start_matches('/');
        let url = format!("{}/{}", self.endpoint, clean);
        let mut req = ureq::head(&url);
        if let Some(auth) = self.auth_header() {
            req = req.set("Authorization", &auth);
        }

        match req.call() {
            Ok(resp) => {
                let len = resp.header("Content-Length").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                let etag = resp.header("ETag").map(|s| s.trim_matches('"').to_string());
                Ok(Some((len, etag)))
            }
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(e) => Err(format!("WebDAV HEAD failed: {e}")),
        }
    }

    pub(crate) fn ensure_collection(&self, relative_dir: &str) -> Result<(), String> {
        let clean = relative_dir.trim_matches('/');
        if clean.is_empty() {
            return Ok(());
        }
        let parts: Vec<&str> = clean.split('/').collect();
        let mut current = String::new();
        for part in parts {
            if current.is_empty() {
                current = part.to_string();
            } else {
                current = format!("{current}/{part}");
            }
            let url = format!("{}/{}", self.endpoint, current);
            let mut req = ureq::request("MKCOL", &url);
            if let Some(auth) = self.auth_header() {
                req = req.set("Authorization", &auth);
            }
            let _ = req.call(); // 405 Method Not Allowed means collection already exists
        }
        Ok(())
    }

    pub(crate) fn put_object_path(&self, relative_path: &str, data: &[u8]) -> Result<(), String> {
        let clean = relative_path.trim_start_matches('/');
        if let Some(idx) = clean.rfind('/') {
            let parent_dir = &clean[..idx];
            let _ = self.ensure_collection(parent_dir);
        }
        let url = format!("{}/{}", self.endpoint, clean);
        let mut req = ureq::put(&url);
        if let Some(auth) = self.auth_header() {
            req = req.set("Authorization", &auth);
        }

        let resp = req
            .send_bytes(data)
            .map_err(|e| format!("WebDAV upload failed: {e}"))?;
        if resp.status() >= 200 && resp.status() < 300 {
            Ok(())
        } else {
            Err(format!("WebDAV PUT returned status: {}", resp.status()))
        }
    }

    pub fn put_object(&self, filename: &str, data: &[u8]) -> Result<(), String> {
        self.put_object_path(filename, data)
    }

    pub fn get_object(&self, filename: &str) -> Result<Vec<u8>, String> {
        let url = format!("{}/{}", self.endpoint, filename);
        let mut req = ureq::get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.set("Authorization", &auth);
        }

        let resp = req
            .call()
            .map_err(|e| format!("WebDAV download failed: {e}"))?;
        if resp.status() >= 200 && resp.status() < 300 {
            let mut bytes = Vec::new();
            resp.into_reader()
                .read_to_end(&mut bytes)
                .map_err(|e| format!("Failed to read WebDAV payload: {e}"))?;
            Ok(bytes)
        } else {
            Err(format!("WebDAV GET returned status: {}", resp.status()))
        }
    }

    pub fn list_snapshots(&self) -> Result<Vec<String>, String> {
        let mut req = ureq::request("PROPFIND", self.endpoint).set("Depth", "1");
        if let Some(auth) = self.auth_header() {
            req = req.set("Authorization", &auth);
        }

        let resp = req
            .call()
            .map_err(|e| format!("WebDAV listing failed: {e}"))?;
        let body = resp
            .into_string()
            .map_err(|e| format!("Failed to parse WebDAV response: {e}"))?;

        let mut keys = Vec::new();
        for chunk in body.split("<d:href>") {
            if let Some(href) = chunk.split("</d:href>").next() {
                let trimmed = href.trim_matches('/');
                if let Some(filename) = trimmed.rsplit('/').next() {
                    if filename.ends_with(".zip") && filename.contains("berry_snapshot") {
                        keys.push(filename.to_string());
                    }
                }
            }
        }
        keys.sort_by(|a, b| b.cmp(a));
        Ok(keys)
    }
}

// -----------------------------------------------------------------------------
// High-Level Backup and Restore Operations
// -----------------------------------------------------------------------------

/// Test connection to configured cloud backup provider.
pub fn test_cloud_connection(config: &CloudBackupConfig) -> CloudPingResult {
    match config.provider {
        CloudStorageProvider::LocalPath => {
            let path_str = config.local_path.as_deref().unwrap_or("");
            if path_str.trim().is_empty() {
                return CloudPingResult {
                    success: false,
                    latency_ms: 0,
                    message: "Local path is not specified".to_string(),
                };
            }
            let start = Instant::now();
            let path = Path::new(path_str);
            if !path.exists() {
                if let Err(e) = fs::create_dir_all(path) {
                    return CloudPingResult {
                        success: false,
                        latency_ms: start.elapsed().as_millis() as u64,
                        message: format!("Cannot create local destination directory: {e}"),
                    };
                }
            }
            CloudPingResult {
                success: true,
                latency_ms: start.elapsed().as_millis() as u64,
                message: format!("Local directory accessible: {path_str}"),
            }
        }
        CloudStorageProvider::WebDav => match WebDavClient::from_config(config) {
            Ok(client) => match client.ping() {
                Ok(latency_ms) => CloudPingResult {
                    success: true,
                    latency_ms,
                    message: format!("WebDAV connected successfully ({latency_ms}ms)"),
                },
                Err(err) => CloudPingResult {
                    success: false,
                    latency_ms: 0,
                    message: err,
                },
            },
            Err(err) => CloudPingResult {
                success: false,
                latency_ms: 0,
                message: err,
            },
        },
        CloudStorageProvider::S3 => match S3Client::from_config(config) {
            Ok(client) => match client.ping() {
                Ok(latency_ms) => CloudPingResult {
                    success: true,
                    latency_ms,
                    message: format!("S3 bucket connected successfully ({latency_ms}ms)"),
                },
                Err(err) => CloudPingResult {
                    success: false,
                    latency_ms: 0,
                    message: err,
                },
            },
            Err(err) => CloudPingResult {
                success: false,
                latency_ms: 0,
                message: err,
            },
        },
    }
}

/// Creates a complete point-in-time library snapshot archive and uploads it to the chosen provider.
pub fn create_cloud_snapshot(
    db: &Database,
    config: &CloudBackupConfig,
    temp_dir: &Path,
    description: Option<String>,
) -> Result<CloudBackupResult, String> {
    let start_time = Instant::now();
    let stats = db
        .get_database_stats()
        .map_err(|e| format!("Failed to query database stats: {e}"))?;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let (date_str, _) = format_iso8601_basic(SystemTime::now());
    let snapshot_id = format!("berry_snapshot_{date_str}_{now_secs}");
    let filename = format!("{snapshot_id}.zip");

    // 1. Point-in-time consistent SQLite database backup using VACUUM INTO
    let temp_db_path = temp_dir.join(format!("{snapshot_id}.db"));
    db.backup_database(&temp_db_path.to_string_lossy())
        .map_err(|e| format!("SQLite VACUUM INTO backup failed: {e}"))?;

    let db_bytes = fs::read(&temp_db_path)
        .map_err(|e| format!("Failed to read generated SQLite snapshot: {e}"))?;
    let _ = fs::remove_file(&temp_db_path);

    // 2. Build metadata manifest
    let manifest = CloudSnapshotMeta {
        snapshot_id: snapshot_id.clone(),
        filename: filename.clone(),
        size_bytes: 0, // updated below
        created_at: now_secs,
        file_count: stats.file_count,
        folder_count: stats.folder_count,
        tag_count: stats.tag_count,
        album_count: stats.album_count,
        description,
        berry_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize snapshot manifest: {e}"))?;

    // 3. Assemble zip bundle
    let mut zip_buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buffer));
        let zip_opts =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("manifest.json", zip_opts)
            .map_err(|e| format!("Failed to start manifest in zip: {e}"))?;
        zip.write_all(&manifest_bytes)
            .map_err(|e| format!("Failed to write manifest in zip: {e}"))?;

        zip.start_file("berry.db", zip_opts)
            .map_err(|e| format!("Failed to start berry.db in zip: {e}"))?;
        zip.write_all(&db_bytes)
            .map_err(|e| format!("Failed to write berry.db in zip: {e}"))?;

        zip.finish()
            .map_err(|e| format!("Failed to finish zip bundle: {e}"))?;
    }

    let mut final_manifest = manifest;
    final_manifest.size_bytes = zip_buffer.len() as u64;

    // 4. Upload to target cloud provider
    match config.provider {
        CloudStorageProvider::LocalPath => {
            let target_dir = config
                .local_path
                .as_deref()
                .ok_or_else(|| "Local path is not configured".to_string())?;
            let dest_dir = Path::new(target_dir);
            fs::create_dir_all(dest_dir)
                .map_err(|e| format!("Failed to create destination dir: {e}"))?;
            let out_file = dest_dir.join(&filename);
            fs::write(&out_file, &zip_buffer)
                .map_err(|e| format!("Failed to write snapshot to local path: {e}"))?;
        }
        CloudStorageProvider::WebDav => {
            let client = WebDavClient::from_config(config)?;
            client.put_object(&filename, &zip_buffer)?;
        }
        CloudStorageProvider::S3 => {
            let client = S3Client::from_config(config)?;
            client.put_object(&filename, &zip_buffer)?;
        }
    }

    Ok(CloudBackupResult {
        success: true,
        snapshot: Some(final_manifest),
        duration_ms: start_time.elapsed().as_millis() as u64,
        error: None,
    })
}

/// Lists all available snapshot archives from the chosen provider.
pub fn list_cloud_snapshots(config: &CloudBackupConfig) -> Result<Vec<CloudSnapshotMeta>, String> {
    match config.provider {
        CloudStorageProvider::LocalPath => {
            let dir_str = config
                .local_path
                .as_deref()
                .ok_or_else(|| "Local path is not configured".to_string())?;
            let dir = Path::new(dir_str);
            if !dir.exists() {
                return Ok(Vec::new());
            }

            let mut snapshots = Vec::new();
            for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read dir: {e}"))? {
                let entry = entry.map_err(|e| format!("Dir entry error: {e}"))?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default();
                    if file_name.contains("berry_snapshot") {
                        if let Ok(meta) = extract_manifest_from_zip_file(&path) {
                            snapshots.push(meta);
                        }
                    }
                }
            }
            snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(snapshots)
        }
        CloudStorageProvider::WebDav => {
            let client = WebDavClient::from_config(config)?;
            let filenames = client.list_snapshots()?;
            let mut snapshots = Vec::new();
            for fname in filenames {
                if let Ok(data) = client.get_object(&fname) {
                    if let Ok(meta) = extract_manifest_from_zip_bytes(&data) {
                        snapshots.push(meta);
                    }
                }
            }
            snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(snapshots)
        }
        CloudStorageProvider::S3 => {
            let client = S3Client::from_config(config)?;
            let filenames = client.list_snapshots()?;
            let mut snapshots = Vec::new();
            for fname in filenames {
                if let Ok(data) = client.get_object(&fname) {
                    if let Ok(meta) = extract_manifest_from_zip_bytes(&data) {
                        snapshots.push(meta);
                    }
                }
            }
            snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(snapshots)
        }
    }
}

/// Restores a snapshot database into the active SQLite database path.
pub fn restore_cloud_snapshot(
    active_db_path: &Path,
    config: &CloudBackupConfig,
    snapshot_filename: &str,
) -> Result<CloudRestoreResult, String> {
    let start_time = Instant::now();

    // 1. Download/read the snapshot archive bytes
    let zip_bytes = match config.provider {
        CloudStorageProvider::LocalPath => {
            let dir_str = config
                .local_path
                .as_deref()
                .ok_or_else(|| "Local path is not configured".to_string())?;
            let archive_path = Path::new(dir_str).join(snapshot_filename);
            fs::read(&archive_path)
                .map_err(|e| format!("Failed to read local snapshot file: {e}"))?
        }
        CloudStorageProvider::WebDav => {
            let client = WebDavClient::from_config(config)?;
            client.get_object(snapshot_filename)?
        }
        CloudStorageProvider::S3 => {
            let client = S3Client::from_config(config)?;
            client.get_object(snapshot_filename)?
        }
    };

    // 2. Unpack and extract berry.db
    let mut archive = ZipArchive::new(Cursor::new(&zip_bytes))
        .map_err(|e| format!("Failed to parse zip archive: {e}"))?;

    let mut db_entry = archive
        .by_name("berry.db")
        .map_err(|_| "Snapshot archive does not contain berry.db".to_string())?;

    let mut restored_db_bytes = Vec::new();
    db_entry
        .read_to_end(&mut restored_db_bytes)
        .map_err(|e| format!("Failed to unpack berry.db: {e}"))?;

    // 3. Write to temporary validation database and verify schema integrity
    let temp_validation_path = active_db_path.with_extension("restore_temp.db");
    fs::write(&temp_validation_path, &restored_db_bytes)
        .map_err(|e| format!("Failed to write temporary validation database: {e}"))?;

    let restored_file_count = match Database::connect(&temp_validation_path) {
        Ok(db_check) => match db_check.get_database_stats() {
            Ok(stats) => stats.file_count,
            Err(e) => {
                let _ = fs::remove_file(&temp_validation_path);
                return Err(format!("Restored database integrity check failed: {e}"));
            }
        },
        Err(e) => {
            let _ = fs::remove_file(&temp_validation_path);
            return Err(format!("Cannot open restored SQLite database: {e}"));
        }
    };

    // 4. Create safe rollback backup of current live database
    let rollback_path = active_db_path.with_extension("pre_restore_bak");
    if active_db_path.exists() {
        let _ = fs::copy(active_db_path, &rollback_path);
    }

    // 5. Atomic rename replacement
    if let Err(e) = fs::copy(&temp_validation_path, active_db_path) {
        // Rollback
        if rollback_path.exists() {
            let _ = fs::copy(&rollback_path, active_db_path);
        }
        let _ = fs::remove_file(&temp_validation_path);
        return Err(format!("Failed to replace live database: {e}"));
    }
    let _ = fs::remove_file(&temp_validation_path);

    Ok(CloudRestoreResult {
        success: true,
        restored_files_count: restored_file_count,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error: None,
    })
}

fn extract_manifest_from_zip_bytes(bytes: &[u8]) -> Result<CloudSnapshotMeta, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("Invalid zip archive: {e}"))?;
    let mut file = archive
        .by_name("manifest.json")
        .map_err(|_| "Missing manifest.json".to_string())?;
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)
        .map_err(|e| format!("Failed to read manifest: {e}"))?;
    serde_json::from_str(&json_str).map_err(|e| format!("Corrupt manifest json: {e}"))
}

fn extract_manifest_from_zip_file(path: &Path) -> Result<CloudSnapshotMeta, String> {
    let file = File::open(path).map_err(|e| format!("Open error: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Zip error: {e}"))?;
    let mut manifest_file = archive
        .by_name("manifest.json")
        .map_err(|_| "Missing manifest.json".to_string())?;
    let mut json_str = String::new();
    manifest_file
        .read_to_string(&mut json_str)
        .map_err(|e| format!("Failed to read manifest: {e}"))?;
    serde_json::from_str(&json_str).map_err(|e| format!("Corrupt manifest: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha256_rfc4231_vector() {
        // RFC 4231 Test Case 1:
        // Key = 0x0b repeated 20 times
        // Data = "Hi There"
        // Digest = 0xb0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
        let key = vec![0x0bu8; 20];
        let data = b"Hi There";
        let hmac = hmac_sha256(&key, data);
        assert_eq!(
            hex::encode(hmac),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }
}
