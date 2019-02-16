use std::sync::Arc;
use std::sync::Mutex;

use redis::{Client, Connection};

use error::MainError;

#[allow(dead_code)]
pub struct AppState {
    client: Arc<Mutex<Client>>,
    connection: Arc<Mutex<Connection>>,
    config: Arc<Mutex<AppConfig>>,
}

#[allow(dead_code)]
impl AppState {
    pub fn new(
        client: &Arc<Mutex<Client>>,
        connection: &Arc<Mutex<Connection>>,
        config: &Arc<Mutex<AppConfig>>,
    ) -> AppState {
        AppState {
            client: client.clone(),
            connection: connection.clone(),
            config: config.clone(),
        }
    }

    pub fn get_config(&self) -> Arc<Mutex<AppConfig>> {
        self.config.clone()
    }

    pub fn get_client(&self) -> Arc<Mutex<Client>> {
        self.client.clone()
    }

    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.connection.clone()
    }

    pub fn reconnect(&self) -> Result<Arc<Mutex<Connection>>, MainError> {
        let client_mutex = self.get_client();
        let connection_mutex = self.get_connection();

        let client = &*client_mutex.lock().unwrap();
        let mut connection = connection_mutex.lock().unwrap();

        *connection = client.get_connection()?;

        Ok(self.get_connection())
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,
    pub db_index: i64,
}
