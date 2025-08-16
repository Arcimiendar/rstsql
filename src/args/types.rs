use clap::Parser;


fn validate_bind_address(addr: &str) -> Result<String, String> {
    if addr.parse::<std::net::IpAddr>().is_ok() {
        Ok(addr.to_string())
    } else {
        Err(format!("Invalid bind address: {}", addr))
    }
}

fn validate_log_config(path: &str) -> Result<String, String> {
    let path_obj = std::path::Path::new(path);
    if path_obj.exists() && path_obj.is_file() {
        Ok(path.to_string())
    } else {
        Err(format!("Log configuration file does not exist: {}", path))
    }
}

fn validate_dsl_path(path: &str) -> Result<String, String> {
    let path_obj = std::path::Path::new(path);
    if path_obj.exists() && path_obj.is_dir() {
        Ok(path.to_string())
    } else {
        Err(format!("DSL path does not exist: {}", path))
    }
}


#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The port to run the server on
    #[arg(short, long, env, default_value = "8080", value_parser = clap::value_parser!(u16).range(1..65535))]
    pub port: u16,

    /// Bind address for the server
    #[arg(short, long, env, default_value = "127.0.0.1", value_parser = validate_bind_address)]
    pub bind: String,

    /// Logging configuration file
    #[arg(short, long, env, default_value = None, value_parser = validate_log_config)]
    pub log_config: Option<String>,

    /// Path to the DSL files
    #[arg(short, long, env, default_value = "/DSL", value_parser = validate_dsl_path)]
    pub dsl_path: String,

    #[arg(long, env, short = 'D', default_value = "postgresql://test:01234@127.0.0.1:5432/test")]
    pub db_uri: String,
}

pub fn get_args() -> Args {
    Args::parse()
}