use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::collections::HashMap;
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub generator_model: String,
    pub refiner_model: String,
    pub validator_model: String,
    pub curator_model: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeys {
    pub openrouter_key: Option<String>,
    pub hive_key: Option<String>,
    pub anthropic_key: Option<String>,
}

impl Default for ApiKeys {
    fn default() -> Self {
        Self {
            openrouter_key: None,
            hive_key: None,
            anthropic_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_accept: bool,
    pub theme: String,
    pub font_size: u32,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug)]
pub struct AppState {
    pub profiles: Arc<RwLock<Vec<Profile>>>,
    pub active_profile: Arc<RwLock<Option<String>>>,
    pub analytics_data: Arc<RwLock<AnalyticsData>>,
    pub api_keys: Arc<RwLock<ApiKeys>>,
    pub settings: Arc<RwLock<Settings>>,
    pub consensus_engine: Arc<Mutex<Option<ConsensusEngineHandle>>>,
    pub database: Arc<Mutex<Connection>>,
    pub cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub total_queries: u32,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub success_rate: f32,
    pub avg_response_time: f32,
    pub queries_by_day: HashMap<String, u32>,
    pub cost_by_model: HashMap<String, f64>,
}

#[derive(Debug)]
pub struct ConsensusEngineHandle {
    pub sender: mpsc::UnboundedSender<ConsensusCommand>,
    pub handle: tokio::task::JoinHandle<()>,
}

#[derive(Debug)]
pub enum ConsensusCommand {
    RunQuery { query: String, profile_id: String },
    Cancel,
    UpdateProfile { profile: Profile },
}

#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub fn cancel(&self) {
        self.cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl AppState {
    pub fn new() -> Self {
        // Initialize database
        let db_path = Self::get_database_path();
        let database = Connection::open(&db_path).expect("Failed to open database");
        
        // Initialize database schema
        Self::init_database_schema(&database).expect("Failed to init database");
        
        // Load profiles from database
        let profiles = Self::load_profiles_from_db(&database).unwrap_or_else(|_| Self::default_profiles());
        
        // Load API keys from database
        let api_keys = Self::load_api_keys_from_db(&database).unwrap_or_default();
        
        // Load settings from database
        let settings = Self::load_settings_from_db(&database).unwrap_or_else(|_| Settings {
            auto_accept: false,
            theme: "dark".to_string(),
            font_size: 14,
            show_line_numbers: true,
            word_wrap: false,
            max_tokens: 4096,
            temperature: 0.7,
        });
        
        Self {
            profiles: Arc::new(RwLock::new(profiles)),
            active_profile: Arc::new(RwLock::new(Some("default".to_string()))),
            analytics_data: Arc::new(RwLock::new(AnalyticsData {
                total_queries: 0,
                total_cost: 0.0,
                total_tokens: 0,
                success_rate: 98.5,
                avg_response_time: 2.3,
                queries_by_day: HashMap::new(),
                cost_by_model: HashMap::new(),
            })),
            api_keys: Arc::new(RwLock::new(api_keys)),
            settings: Arc::new(RwLock::new(settings)),
            consensus_engine: Arc::new(Mutex::new(None)),
            database: Arc::new(Mutex::new(database)),
            cancellation_token: Arc::new(Mutex::new(None)),
        }
    }
    
    fn get_database_path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not find home directory");
        let hive_dir = home.join(".hive");
        std::fs::create_dir_all(&hive_dir).expect("Could not create .hive directory");
        hive_dir.join("hive.db")
    }
    
    fn init_database_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                generator_model TEXT NOT NULL,
                refiner_model TEXT NOT NULL,
                validator_model TEXT NOT NULL,
                curator_model TEXT NOT NULL,
                is_default INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS api_keys (
                key_type TEXT PRIMARY KEY,
                key_value TEXT
            );
            
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                response TEXT,
                profile_id TEXT,
                cost REAL,
                tokens INTEGER,
                duration_ms INTEGER,
                created_at TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS analytics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                metric_type TEXT NOT NULL,
                metric_value TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
        "#)?;
        Ok(())
    }
    
    fn load_profiles_from_db(conn: &Connection) -> Result<Vec<Profile>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, generator_model, refiner_model, validator_model, curator_model, is_default, created_at 
             FROM profiles ORDER BY created_at"
        )?;
        
        let profiles = stmt.query_map([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                generator_model: row.get(2)?,
                refiner_model: row.get(3)?,
                validator_model: row.get(4)?,
                curator_model: row.get(5)?,
                is_default: row.get::<_, i32>(6)? == 1,
                created_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        if profiles.is_empty() {
            Ok(Self::default_profiles())
        } else {
            Ok(profiles)
        }
    }
    
    fn load_api_keys_from_db(conn: &Connection) -> Result<ApiKeys> {
        let mut keys = ApiKeys::default();
        
        let mut stmt = conn.prepare("SELECT key_type, key_value FROM api_keys")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        for row in rows {
            if let Ok((key_type, key_value)) = row {
                match key_type.as_str() {
                    "openrouter" => keys.openrouter_key = Some(key_value),
                    "hive" => keys.hive_key = Some(key_value),
                    "anthropic" => keys.anthropic_key = Some(key_value),
                    _ => {}
                }
            }
        }
        
        Ok(keys)
    }
    
    fn load_settings_from_db(conn: &Connection) -> Result<Settings> {
        let mut settings = Settings {
            auto_accept: false,
            theme: "dark".to_string(),
            font_size: 14,
            show_line_numbers: true,
            word_wrap: false,
            max_tokens: 4096,
            temperature: 0.7,
        };
        
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        for row in rows {
            if let Ok((key, value)) = row {
                match key.as_str() {
                    "auto_accept" => settings.auto_accept = value == "true",
                    "theme" => settings.theme = value,
                    "font_size" => settings.font_size = value.parse().unwrap_or(14),
                    "show_line_numbers" => settings.show_line_numbers = value == "true",
                    "word_wrap" => settings.word_wrap = value == "true",
                    "max_tokens" => settings.max_tokens = value.parse().unwrap_or(4096),
                    "temperature" => settings.temperature = value.parse().unwrap_or(0.7),
                    _ => {}
                }
            }
        }
        
        Ok(settings)
    }
    
    fn default_profiles() -> Vec<Profile> {
        vec![
            Profile {
                id: "default".to_string(),
                name: "Default".to_string(),
                generator_model: "gpt-4".to_string(),
                refiner_model: "gpt-3.5-turbo".to_string(),
                validator_model: "gpt-3.5-turbo".to_string(),
                curator_model: "gpt-4".to_string(),
                is_default: true,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            Profile {
                id: "speed".to_string(),
                name: "Speed".to_string(),
                generator_model: "gpt-3.5-turbo".to_string(),
                refiner_model: "gpt-3.5-turbo".to_string(),
                validator_model: "gpt-3.5-turbo".to_string(),
                curator_model: "gpt-3.5-turbo".to_string(),
                is_default: false,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            Profile {
                id: "quality".to_string(),
                name: "Quality".to_string(),
                generator_model: "gpt-4".to_string(),
                refiner_model: "gpt-4".to_string(),
                validator_model: "gpt-4".to_string(),
                curator_model: "gpt-4".to_string(),
                is_default: false,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        ]
    }
    
    pub fn save_api_key(&self, key_type: &str, key_value: &str) -> Result<()> {
        let db = self.database.lock();
        db.execute(
            "INSERT OR REPLACE INTO api_keys (key_type, key_value) VALUES (?1, ?2)",
            [key_type, key_value],
        )?;
        
        // Update in-memory state
        let mut keys = self.api_keys.write();
        match key_type {
            "openrouter" => keys.openrouter_key = Some(key_value.to_string()),
            "hive" => keys.hive_key = Some(key_value.to_string()),
            "anthropic" => keys.anthropic_key = Some(key_value.to_string()),
            _ => {}
        }
        
        Ok(())
    }
    
    pub fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        let db = self.database.lock();
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
        
        // Update in-memory state
        let mut settings = self.settings.write();
        match key {
            "auto_accept" => settings.auto_accept = value == "true",
            "theme" => settings.theme = value.to_string(),
            "font_size" => settings.font_size = value.parse().unwrap_or(14),
            "show_line_numbers" => settings.show_line_numbers = value == "true",
            "word_wrap" => settings.word_wrap = value == "true",
            "max_tokens" => settings.max_tokens = value.parse().unwrap_or(4096),
            "temperature" => settings.temperature = value.parse().unwrap_or(0.7),
            _ => {}
        }
        
        Ok(())
    }
    
    pub fn save_profile(&self, profile: &Profile) -> Result<()> {
        let db = self.database.lock();
        db.execute(
            "INSERT OR REPLACE INTO profiles (id, name, generator_model, refiner_model, validator_model, curator_model, is_default, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &profile.id,
                &profile.name,
                &profile.generator_model,
                &profile.refiner_model,
                &profile.validator_model,
                &profile.curator_model,
                if profile.is_default { 1 } else { 0 },
                &profile.created_at,
            ),
        )?;
        
        // Update in-memory state
        let mut profiles = self.profiles.write();
        if let Some(existing) = profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile.clone();
        } else {
            profiles.push(profile.clone());
        }
        
        Ok(())
    }
    
    pub fn record_conversation(&self, query: &str, response: &str, profile_id: &str, cost: f64, tokens: u32, duration_ms: u64) -> Result<()> {
        let db = self.database.lock();
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        
        db.execute(
            "INSERT INTO conversations (id, query, response, profile_id, cost, tokens, duration_ms, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                id,
                query,
                response,
                profile_id,
                cost,
                tokens as i64,
                duration_ms as i64,
                created_at,
            ),
        )?;
        
        // Update analytics
        let mut analytics = self.analytics_data.write();
        analytics.total_queries += 1;
        analytics.total_cost += cost;
        analytics.total_tokens += tokens;
        
        // Update daily stats
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        *analytics.queries_by_day.entry(today).or_insert(0) += 1;
        
        Ok(())
    }
}