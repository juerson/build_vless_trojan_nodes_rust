mod file_data;

use file_data::MyData;
use lazy_static::lazy_static;
use rand::{ prelude::{ IndexedRandom, SliceRandom }, Rng };
use serde::Deserialize;
use std::{ fs::{ self, File }, io::{ BufWriter, Write }, path::Path };
use urlencoding::encode;
use base64::{ Engine as _, engine::general_purpose };
use clap::{ CommandFactory, Parser, ValueEnum };

lazy_static! {
    static ref PORTS_ARRAY_HTTP: [u16; 7] = [80, 8080, 8880, 2052, 2082, 2086, 2095];
    static ref PORTS_ARRAY_HTTPS: [u16; 6] = [443, 2053, 2083, 2087, 2096, 8443];
}

/// 功能：用于构建cf节点的分享链接，支持vless/trojan+ws[+tls]、shadowsocks+v2ray-plugin+websocket[+tls]。
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// 配置文件
    #[arg(short = 'f', value_name = "config_path", default_value = "config.yaml")]
    config_path: String,

    /// 输入的数据文件(数据是ip、域名的txt/csv文件)
    #[arg(short = 'i', value_name = "data_path", default_value = "result.csv")]
    data_path: String,

    /// 输出的结果文件
    #[arg(short = 'o', value_name = "output_path", default_value = "output.txt")]
    output_path: String,

    /// 从数据文件中，最大读取数
    #[arg(short = 'n', value_name = "count", default_value_t = 300)]
    count: usize,

    /// 选择指定的代理类型，可选值：vless,trojan,ss
    #[arg(short = 's', value_name = "selected_type", default_value = "")]
    selected_type: String,

    /// 排除不要的代理类型，可选值：vless,trojan,ss
    #[arg(short = 'e', value_name = "excluded_type", default_value = "")]
    excluded_type: String,

    /// 如果是csv数据文件，采用哪列数据作为节点别名的一部分，可选值：colo,loc,region,city
    #[arg(short = 'c', value_name = "column_name", default_value = "colo")]
    csv_column_name: String,

    /// 选择哪个TLS模式，只添加该参数不带值则是true值。可选值：none,true,false
    #[arg(long, value_name = "tls", default_missing_value = "true", num_args = 0..=1)]
    tls: Option<ActiveTls>,
}

#[derive(Debug, Clone, ValueEnum)]
enum ActiveTls {
    #[clap(alias = "none")]
    None,
    True,
    False,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    #[serde(rename = "type")]
    proxy_type: String,
    uuid: Option<String>,
    password: Option<String>,
    cipher: Option<String>,
    host: Option<String>,
    sni: Option<String>,
    path: Option<String>,
    tls: Option<bool>,
    alpn: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::try_parse().unwrap_or_else(|_err| {
        // 打印帮助信息
        Args::command().print_help().unwrap();
        println!();
        // 退出程序，状态码 1 表示错误
        std::process::exit(1);
    });

    // 路径校验、文件校验、文件为空校验
    let path_vec = vec![("Config File", &cli.config_path), ("Data File", &cli.data_path)];
    for (label, path_str) in path_vec {
        validate_non_empty_file(label, path_str);
    }

    let config_path = cli.config_path;
    let data_path = cli.data_path;
    let output_path = cli.output_path;
    let max_data_count = cli.count;
    let selected_type = cli.selected_type;
    let excluded_type = cli.excluded_type;
    let tls_cli_value: Option<ActiveTls> = cli.tls;
    let csv_column_name = cli.csv_column_name;

    let configs: Vec<Config> = serde_yaml::from_slice(&std::fs::read(config_path)?)?;
    if configs.len() > 0 {
        let data_vec: Vec<MyData> = file_data::process_files_data(
            &csv_column_name, // colo(数据中心),loc(国家代码),region(地区),city(城市)
            0,
            max_data_count,
            &data_path
        );
        let results = process_data(
            configs,
            data_vec,
            &selected_type,
            &excluded_type,
            tls_cli_value
        );
        // 写入文件
        let file = File::create(output_path)?;
        let mut buf_writer = BufWriter::new(file);
        for result in results {
            writeln!(buf_writer, "{}", result)?;
        }
        buf_writer.flush()?; // 显示刷新缓冲区
    }

    Ok(())
}

/// 处理数据并构建节点
fn process_data(
    configs: Vec<Config>,
    datas: Vec<MyData>,
    selected_type: &str,
    excluded_type: &str,
    tls_cli_value: Option<ActiveTls>
) -> Vec<String> {
    let configs_len = configs.len();
    let mut results = Vec::new();

    for data in datas {
        let address = data.addr;
        let mut port = data.port.unwrap_or(0);
        let alias = data.alias.unwrap_or_default();

        // 下面的id是指yaml配置中哪个数组的元素？(已经index+1)
        if
            let Some((id, config, tls_value)) = (0..100).find_map(|_| {
                let idx = rand::rng().random_range(..configs_len);
                let config = configs[idx].clone();
                let ctype = config.clone().proxy_type; // 选配置中哪个节点(代理类型)

                // 选中的代理类型
                let selected_type_bool = match selected_type {
                    "vless" => ctype == "vless",
                    "trojan" => ctype == "trojan",
                    "ss" => ctype == "ss",
                    _ => true,
                };

                // 选择合适的tls模式的配置
                let tls_config_value = config.tls.unwrap_or(true);
                let tls_filter: bool = match tls_cli_value {
                    None | Some(ActiveTls::None) => true,
                    Some(ActiveTls::True) => tls_config_value,
                    Some(ActiveTls::False) => !tls_config_value,
                };

                if ctype != excluded_type && selected_type_bool && tls_filter {
                    Some((idx, config, tls_config_value)) // 这三个值返回，交给if条件里面的代码块使用
                } else {
                    None
                }
            })
        {
            // println!("✅ 满足条件的 id: {}", id + 1);
            // 根据TLS值，强行修改端口不匹配的
            match tls_value {
                true => {
                    let mut ports = PORTS_ARRAY_HTTPS.to_vec();
                    ports.shuffle(&mut rand::rng()); // 打乱顺序
                    if let Some(element) = ports.choose(&mut rand::rng()) {
                        if port == 0 || PORTS_ARRAY_HTTP.contains(&port) {
                            port = *element;
                        }
                    }
                }
                false => {
                    let mut ports = PORTS_ARRAY_HTTP.to_vec();
                    ports.shuffle(&mut rand::rng()); // 打乱顺序
                    if let Some(element) = PORTS_ARRAY_HTTP.choose(&mut rand::rng()) {
                        if port == 0 || PORTS_ARRAY_HTTPS.contains(&port) {
                            port = *element;
                        }
                    }
                }
            }

            // 下面定义节点的别名
            let width = ((configs_len as f64).log10().floor() as usize) + 1;
            let index = format!("{:0width$}", id + 1, width = width);
            let remarks = match alias.is_empty() {
                true => format!("【{}】{}:{}", index, address, port),
                false => format!("【{}】{} | {}:{}", index, alias, address, port),
            };

            if config.proxy_type == "vless" {
                let uuid = config.uuid.unwrap_or_default();
                let host = config.host.unwrap_or_default();
                let sni = config.sni.unwrap_or_default();
                let path = config.path.unwrap_or_default();
                let alpn = config.alpn.unwrap_or("h3".to_string());
                let insert_security_str = match tls_value {
                    true =>
                        &format!(
                            "tls&sni={}&alpn={}&fp=chrome&allowInsecure=1",
                            sni,
                            encode(&alpn)
                        ),
                    false => "none",
                };
                let vless_link = format!(
                    "vless://{}@{}:{}?encryption=none&security={}&type=ws&host={}&path={}#{}",
                    uuid,
                    address,
                    port,
                    insert_security_str,
                    host,
                    encode(&path),
                    encode(&remarks)
                );
                results.push(vless_link);
            } else if config.proxy_type == "trojan" {
                let password = config.password.unwrap_or_default();
                let host = config.host.unwrap_or_default();
                let sni = config.sni.unwrap_or_default();
                let path = config.path.unwrap_or_default();
                let alpn = config.alpn.unwrap_or("h3".to_string());
                let insert_security_str = match tls_value {
                    true =>
                        &format!(
                            "tls&sni={}&alpn={}&fp=chrome&allowInsecure=1",
                            sni,
                            encode(&alpn)
                        ),
                    false => "none",
                };
                let trojan_link = format!(
                    "trojan://{}@{}:{}?security={}&type=ws&host={}&path={}#{}",
                    password,
                    address,
                    port,
                    insert_security_str,
                    host,
                    encode(&path),
                    encode(&remarks)
                );
                results.push(trojan_link);
            } else if config.proxy_type == "ss" {
                let password = config.password.unwrap_or("none".to_string());
                let host = config.host.unwrap_or_default();
                let path = config.path.unwrap_or_default();
                let cipher = config.cipher.unwrap_or("none".to_string());
                let base64_encoded = general_purpose::STANDARD.encode(
                    format!("{}:{}", cipher, password).as_bytes()
                );
                let insert_tls_str = match tls_value {
                    true => "tls;",
                    false => "",
                };
                let plugin = format!(
                    "v2ray-plugin;{}mux%3D0;mode%3Dwebsocket;path%3D{};host%3D{}", // "%3D"等价于"="
                    insert_tls_str,
                    path,
                    host
                );
                let ss_link = format!(
                    "ss://{}@{}:{}?plugin={}#{}",
                    base64_encoded,
                    address,
                    port,
                    plugin,
                    encode(&remarks)
                );
                results.push(ss_link);
            }
        } else {
            println!("❌ 100 次内都没有满足条件的");
        }
    }
    results
}

/// 校验文件存在、是文件、非空，否则退出
fn validate_non_empty_file(label: &str, path_str: &str) {
    let path = Path::new(path_str);

    if !path.exists() || !path.is_file() {
        eprintln!("Error: {} '{}' is not a valid file.", label, path_str);
        std::process::exit(1);
    }

    if let Ok(metadata) = fs::metadata(path) {
        if metadata.len() == 0 {
            eprintln!("Error: {} '{}' is an empty file.\n", label, path_str);
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: Cannot access metadata of {} '{}'.\n", label, path_str);
        std::process::exit(1);
    }
}
