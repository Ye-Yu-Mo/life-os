use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_map(std::env::vars())
    }

    pub fn from_env_map<I, K, V>(vars: I) -> Result<Self, AppError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let env = vars
            .into_iter()
            .map(|(key, value)| (key.as_ref().to_string(), value.as_ref().to_string()))
            .collect::<std::collections::HashMap<_, _>>();

        let host = env
            .get("APP_HOST")
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = env
            .get("APP_PORT")
            .map(|value| value.parse::<u16>())
            .transpose()
            .map_err(|error| AppError::Config(format!("invalid APP_PORT: {error}")))?
            .unwrap_or(3000);
        let database_url = env
            .get("DATABASE_URL")
            .cloned()
            .ok_or_else(|| AppError::Config("missing DATABASE_URL".to_string()))?;
        let max_connections = env
            .get("DATABASE_MAX_CONNECTIONS")
            .map(|value| value.parse::<u32>())
            .transpose()
            .map_err(|error| AppError::Config(format!("invalid DATABASE_MAX_CONNECTIONS: {error}")))?
            .unwrap_or(5);

        Ok(Self {
            server: ServerConfig { host, port },
            database: DatabaseConfig {
                url: database_url,
                max_connections,
            },
        })
    }
}
