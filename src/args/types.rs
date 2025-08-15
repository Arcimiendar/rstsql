use clap::Parser;


fn validate_bind_address(addr: &str) -> Result<String, String> {
    if addr.parse::<std::net::IpAddr>().is_ok() {
        Ok(addr.to_string())
    } else {
        Err(format!("Invalid bind address: {}", addr))
    }
}

fn validate_log_config(path: &str) -> Result<String, String> {
    if std::path::Path::new(path).exists() {
        Ok(path.to_string())
    } else {
        Err(format!("Log configuration file does not exist: {}", path))
    }
}


#[derive(Parser, Debug)]
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
}

pub fn get_args() -> Args {
    Args::parse()
}