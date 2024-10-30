use anyhow::{anyhow, Result};

static DEFAULT_DB_USERNAME: &str = "myuser";
static DEFAULT_DB_PASSWORD: &str = "mypass";
static DEFAULT_DB_ROOT_PASS: &str = "myrootpass";
static DEFAULT_DB_DATABASE: &str = "mydatabase";

pub struct DbParams {
    pub ports: Vec<u16>,
    pub mount_cont: String,
    pub username: Option<(String, String)>,
    pub pass: Option<(String, String)>,
    pub root_pass: Option<(String, String)>,
    pub database: Option<(String, String)>,
    pub image_name: String,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
}

// Указан Result с конкретным типом DbParams
pub fn get_db_params(db_type: String) -> Result<DbParams> {
    match db_type.as_str() {
        "pg" | "postgres" => Ok(DbParams {
            ports: vec![5432],
            mount_cont: "/var/lib/postgresql/data".to_string(),
            username: Some(("POSTGRES_USER".to_string(), DEFAULT_DB_USERNAME.to_string())),
            pass: Some((
                "POSTGRES_PASSWORD".to_string(),
                DEFAULT_DB_PASSWORD.to_string(),
            )),
            root_pass: None,
            database: Some(("POSTGRES_DB".to_string(), DEFAULT_DB_DATABASE.to_string())),
            image_name: "postgres".to_string(),
            args: vec![],
            envs: vec![],
        }),

        "mysql" => Ok(DbParams {
            ports: vec![3306],
            mount_cont: "/var/lib/mysql".to_string(),
            username: Some(("MYSQL_USER".to_string(), DEFAULT_DB_USERNAME.to_string())),
            pass: Some((
                "MYSQL_PASSWORD".to_string(),
                DEFAULT_DB_PASSWORD.to_string(),
            )),
            root_pass: Some((
                "MYSQL_ROOT_PASSWORD".to_string(),
                DEFAULT_DB_ROOT_PASS.to_string(),
            )),
            database: Some((
                "MYSQL_DATABASE".to_string(),
                DEFAULT_DB_DATABASE.to_string(),
            )),
            image_name: "mysql".to_string(),
            args: vec![],
            envs: vec![],
        }),
        _ => Err(anyhow!("there is no db type: {}", db_type)),
    }
}

pub struct GetDBConnParams {
    pub from: String,
    pub host: String,
    pub username: Option<String>,
    pub pass: Option<String>,
    pub root_pass: Option<String>,
    pub database: Option<String>,
    pub port: u16,
}

pub fn get_db_conn_str(params: GetDBConnParams) -> Option<String> {
    match params.from.as_str() {
        "pg" | "postgres" => Some(format!(
            "postgresql://{}:{}@{}:{}/{}",
            params
                .username
                .unwrap_or_else(|| DEFAULT_DB_USERNAME.to_string()),
            params
                .pass
                .unwrap_or_else(|| DEFAULT_DB_PASSWORD.to_string()),
            params.host,
            params.port,
            params
                .database
                .unwrap_or_else(|| DEFAULT_DB_DATABASE.to_string())
        )),
        "mysql" => Some(format!(
            "mysql://{}:{}@{}:{}/{}",
            params
                .username
                .unwrap_or_else(|| DEFAULT_DB_USERNAME.to_string()),
            params
                .pass
                .unwrap_or_else(|| DEFAULT_DB_PASSWORD.to_string()),
            params.host,
            params.port,
            params
                .database
                .unwrap_or_else(|| DEFAULT_DB_DATABASE.to_string())
        )),
        _ => None,
    }
}
