use indexmap::IndexSet;
use ipnetwork::IpNetwork;
use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::Path,
    thread,
    time::Duration,
};
use urlencoding::encode;

fn main() {
    let files = vec!["config.json", "ip.txt"];
    let file = File::open(files[0]).expect("Failed to open file");
    let conf: serde_json::Value = serde_json::from_reader(file).expect("Failed to parse JSON");
    /* 获取json中字段的值 */
    let v_password = conf["trojan"]["password"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let v_host = conf["trojan"]["host"].as_str().unwrap_or("").to_string();
    let v_sni = conf["trojan"]["sni"].as_str().unwrap_or("").to_string();
    let v_path = if let Some(value) = conf["trojan"].get("path") {
        if !value.is_null() && !value.as_str().unwrap_or_default().is_empty() {
            value.as_str().unwrap().to_string()
        } else {
            "/".to_string()
        }
    } else {
        "/".to_string()
    };
    // ip.txt文件自检，文件不存在或为空时退出程序
    if !is_file_existing_and_non_empty(files[1]) {
        println!("文件{}不存在或者文件为空！", files[1]);
        println!("{}", "+".repeat(50));
        println!("本程序支持的内容格式如下：");
        println!("192.168.1.1");
        println!("192.168.1.0/24");
        println!("192.168.1.1 443");
        println!("192.168.1.1,443");
        println!("192.168.1.1,443,字段a,字段b,字段c,...");
        println!("time.cloudflare.com");
        println!("ip.sb 2053");
        println!("{}", "+".repeat(50));
        wait_for_enter();
        std::process::exit(1);
    }
    println!(
        "本程序的用途：使用TXT文件中的地址和端口，组成新的Trojan链接，支持批量扩展无数条Trojan链接\n温馨提示：程序执行时，需要额外的依赖文件有：{}",
        format!(r#""{}"、"{}""#, files[0],files[1])
    );
    println!("{:->87}", "");

    // 提示文本
    let hint_text = "默认端口设置的一些建议:
					 【1】非TLS模式，没有个人域名，可以设置端口：80，8080，8880，2052，2082，2086，2095
					 【2】是TLS模式，拥有个人域名，可以设置端口：443，2053，2083，2087，2096，8443
                     【3】当然，您也可以设置其它端口，生成的链接是否可以使用，自己测试！";
    for line in hint_text.lines() {
        println!("{}", line.trim_start());
    }
    // 用途：这个函数返回的结果，判断是否为TLS模式，选择使用哪个链接生成
    let tls_mode = get_tls_from_host(v_host.clone());
    // 选择端口，权限大小：用户输入的端口 > 设置默认端口
    let selected_port = get_default_port_from_host(v_host.clone());
    println!(
        "文件{}中，没有端口时，才使用{}端口！",
        files[1], selected_port
    );
    // 从文件中，按行分割内容，不存在端口，就使用selected_port端口，还有读取到的内容是CIDR就生成IP并添加端口
    let addr_and_port_vec_vec: Vec<Vec<String>> =
        read_addr_with_port_from_file(files[1], &selected_port.to_string());

    println!("{:->87}", "");
    // 节点别名的前缀(由用户选择，是否输入，决定是否使用前缀)
    let alias_prefix: String = get_prefix_from_user_input();
    let result = build_vless_nodes(
        addr_and_port_vec_vec,
        alias_prefix,
        tls_mode,
        v_password,
        v_host,
        v_sni,
        v_path,
    );
    println!("{:->87}", "");
    /* 将结果写入文件中 */
    let result_str = result.join("\n");
    fs::write("output.txt", result_str).expect("Failed to write to file");
    println!("节点已写入到output.txt文件中!");
    println!("\n所有任务执行完毕，2秒后自动关闭窗口...");
    thread::sleep(Duration::from_secs(2));
}

fn build_vless_nodes(
    vec_data: Vec<Vec<String>>,
    alias_prefix: String,
    tls_mode: bool,
    v_password: String,
    v_host: String,
    v_sni: String,
    v_path: String,
) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    let encoded_v_path = encode(&v_path);
    for ip_with_port_vec in vec_data {
        let address = ip_with_port_vec[0].to_string();
        if address.is_empty() {
            continue;
        }
        let port = ip_with_port_vec[1].to_string();
        // 节点的名称
        let alias_name = if alias_prefix.trim().is_empty() {
            address.to_string()
        } else {
            format!("{}_{}", alias_prefix.to_string(), address.to_string()) // 添加节点名称的前缀
        };
        // url编码节点的名称
        let encoded_alias_name = encode(&alias_name);
        let vless;
        if !tls_mode {
            vless = format!(
                "trojan://{}@{}:{}?security=none&type=ws&host={}&path={}#{}",
                v_password, address, port, v_host, encoded_v_path, encoded_alias_name
            )
        } else {
            vless = format!(
                "trojan://{}@{}:{}?security=tls&sni={}&alpn=h3&fp=chrome&type=ws&path={}&host={}#{}",
                v_password, address, port, v_sni, encoded_v_path, v_host, encoded_alias_name
            )
        }
        results.push(vless.clone());
    }
    results
}

/* 读取txt文件的内容，按行读取并分割出地址和端口，最后用嵌套向量搜集结果（Vec<Vec<String>>），结果已经去重了 */
fn read_addr_with_port_from_file(filename: &str, default_port: &str) -> Vec<Vec<String>> {
    let mut result_set: IndexSet<Vec<String>> = IndexSet::new();
    if let Ok(file) = File::open(filename) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(ip_network) = line.trim().parse::<IpNetwork>() {
                    // 是CIDR，就生成IP地址
                    for ip in ip_network.iter() {
                        let mut parts = vec![ip.to_string()];
                        parts.push(default_port.to_string());
                        result_set.insert(parts);
                    }
                } else {
                    // 不是CIDR，使用 split_line_content 函数进行处理
                    let parts = split_line_content(line, default_port);
                    result_set.insert(parts);
                }
            }
        }
    }
    result_set.into_iter().collect()
}

fn split_line_content(line: String, default: &str) -> Vec<String> {
    // 移除左右空白
    let ip_with_port = line.trim().to_string();
    // 分割为单词
    let parts: Cow<str> = if ip_with_port.trim().chars().any(|c| c.is_whitespace()) {
        let splits: Vec<&str> = ip_with_port.trim().split_whitespace().collect();
        Cow::Owned(splits.join(","))
    } else if ip_with_port.trim().contains(',') && ip_with_port.trim().split(',').count() >= 2 {
        let splits: Vec<&str> = ip_with_port
            .trim()
            .split(',')
            .map(|s| s.trim())
            .take(2)
            .collect();
        Cow::Owned(splits.join(","))
    } else {
        let default_parts = vec![ip_with_port.as_str(), default];
        Cow::Owned(default_parts.join(","))
    };

    // 将Cow<str>转换为Vec<String>
    match parts {
        Cow::Borrowed(s) => vec![s.to_string()],
        Cow::Owned(s) => s.split(',').map(String::from).collect(),
    }
}

fn get_default_port_from_host(host_domain: String) -> u16 {
    // default_port这个变量可以修改，如果用户输入端口，就将这个default_port变量的值修改成用户输入的端口
    let mut default_port: u16 =
        if !host_domain.trim().is_empty() && !host_domain.ends_with("workers.dev") {
            443 // 是TLS模式，443，2053，2083，2087，2096，8443
        } else {
            8080 // 非TLS模式，80，8080，8880，2052，2082，2086，2095
        };
    // 获取用户输入的端口，用户输入合法的端口，就修改前面的default_port值
    loop {
        print!(
            "\n这里输入您设置的默认端口【不输入内容，默认使用{}端口】：",
            default_port
        );
        io::stdout().flush().expect("Failed to flush stdout");
        let mut input_port = String::new();
        io::stdin()
            .read_line(&mut input_port)
            .expect("Failed to read line");
        if input_port.trim().is_empty() {
            break;
        }
        match input_port.trim().parse::<u16>() {
            Ok(parsed_port) => {
                if (22..=65535).contains(&parsed_port) {
                    default_port = parsed_port;
                    break;
                } else {
                    // println!("端口号超出范围(22-65535)，请重新输入。");
                }
            }
            Err(_) => {
                // println!("无效的端口号，请重新输入。");
            }
        }
    }
    return default_port;
}

fn get_prefix_from_user_input() -> String {
    print!("【可选】添加Trojan节点的名称前缀或别名前缀，方便自己辨认(默认不添加)：");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

/* ip.txt文件自检（文件是否存在、文件不为空） */
fn is_file_existing_and_non_empty(file_path: &str) -> bool {
    let path = Path::new(file_path);

    if path.exists() {
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_file() {
                let file_size = metadata.len();
                return file_size > 0;
            }
        }
    }
    false
}

fn get_tls_from_host(host_domain: String) -> bool {
    if !host_domain.trim().is_empty() && !host_domain.ends_with("workers.dev") {
        return true; // 是TLS模式
    } else {
        return false; // 不是TLS模式
    };
}

fn wait_for_enter() {
    print!("按Enter键退出程序...");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
}
