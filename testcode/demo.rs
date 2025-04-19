use rand::Rng;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct Config {
    #[serde(rename = "type")]
    proxy_type: String,
    uuid: Option<String>,
    password: Option<String>,
    host: Option<String>,
    sni: Option<String>,
    path: Option<String>,
    tls: Option<bool>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configs: Vec<Config> = serde_yaml::from_slice(&std::fs::read("config.yaml")?)?;
    let mut rng = rand::rng();
    let idx = rng.random_range(..configs.len());
    println!("{}", idx);
    let config = configs[idx].clone();
    if config.proxy_type == "vless" {
        println!("{}", config.uuid.unwrap_or_default());
        println!("{}", config.host.unwrap_or_default());
        println!("{}", config.sni.unwrap_or_default());
        println!("{}", config.path.unwrap_or_default());
    } else if config.proxy_type == "trojan" {
        println!("{}", config.password.unwrap_or_default());
        println!("{}", config.host.unwrap_or_default());
        println!("{}", config.sni.unwrap_or_default());
        println!("{}", config.path.unwrap_or_default());
    } else if config.proxy_type == "ss" {
        println!("{}", config.password.unwrap_or_default());
        println!("{}", config.host.unwrap_or_default());
        println!("{}", config.path.unwrap_or_default());
        println!("{}", config.tls.unwrap_or_default());
    }

    Ok(())
}
