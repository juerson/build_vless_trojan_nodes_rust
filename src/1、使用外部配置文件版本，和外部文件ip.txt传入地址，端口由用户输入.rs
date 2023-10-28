extern crate serde;
use std::fs::File;
use std::io::{self, BufRead};
use std::fs;
use std::thread;
use std::io::Write;
use std::time::Duration;

fn build_vless_nodes(
    list_data: Vec<String>,
    v_port: u16,
    v_host_prefix: &str,
    v_sni: &str,
    v_host: &str,
    v_uuid: &str,
    tls_mode: &str,
) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    let mut vless = String::new();
    for item in list_data {
        let address = item.trim();
        let alias = if v_host_prefix.is_empty() {
            address.to_string()
        } else {
            format!("{}_{}", v_host_prefix, address)
        };
        
        if tls_mode.eq("1") {
           vless = format!(
                "vless://{}@{}:{}?encryption=none&type=ws&host={}&path=%2F%3Fed%3D2048#{}",
                v_uuid, address, v_port, v_host, alias
            )
        };
        if tls_mode.eq("2") {
          vless = format!(
                "vless://{}@{}:{}?encryption=none&security=tls&sni={}&fp=randomized&type=ws&host={}&path=%2F%3Fed%3D2048#{}",
                v_uuid, address, v_port, v_sni, v_host, alias
            )
        } 
        

        results.push(vless.clone());
    }

    results
}

fn main() {
    let file = File::open("config.json").expect("Failed to open file");
    let conf: serde_json::Value = serde_json::from_reader(file).expect("Failed to parse JSON");

    let v_uuid = conf["userID"].as_str().unwrap_or("").to_string();
    let v_host = conf["host"].as_str().unwrap_or("").to_string();
    let v_sni = conf["sni"].as_str().unwrap_or("").to_string();
    let ip_file = File::open("ip.txt").expect("Failed to open file");
    let context_list: Vec<String> = io::BufReader::new(ip_file).lines().map(|x| x.unwrap()).collect();
	println!("批量处理CF Vless节点的小工具(用于生成大量的Vless节点)");
	let large_text = "设置默认的PORT端口的建议:
					 【1】非TLS模式，即没有个人域名的，可以设置：80 8080 8880 2052 2082 2086 2095
					 【2】是TLS模式，即拥有个人域名的，可以设置：443 2053 2083 2087 2096 8443\n";
    for line in large_text.lines() {
        println!("{}", line.trim_start());
    }
	
    let flag = !v_host.ends_with("workers.dev");
    let ports: Vec<u16> = if flag {
		vec![443, 2053, 2083, 2087, 2096, 8443]
    } else {
		vec![80, 8080, 8880, 2052, 2082, 2086, 2095]
    };


    let mut selected_port: u16 = if flag{
        443 // 是TLS模式，默认端口
    } else {
		80 // 非TLS模式，默认端口
    };
    let tls_mode_text = if flag {
          "是TLS模式"
        } else {
          "非TLS模式"
        };
	println!("\n根据你提供的host域名判断，选择{}!",tls_mode_text);
	let tls_mode: String;
	if flag {
		tls_mode = "2".to_string();
	}else{
		tls_mode = "1".to_string();
	};
    println!("\n以下设置默认的端口，可选：{}",ports.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("、"));
	println!("（当然也可以输入其他端口，但不保证生成的vless的链接可用）");
    loop {
		print!("请输入Post端口(默认为：{})：",selected_port);
		let port_input: String = input("");
        if port_input.is_empty() {
            break;
        }
        match port_input.parse::<u16>() {
            Ok(port) => {
                #[allow(unused_comparisons)]
                if 22 < port && port <= 65535 {
                    selected_port = port;
                    println!("正在使用你选择端口号{}，生成Vless节点！", port);
                    break;
                } else {
                    //println!("端口号超出范围(0-65535)，请重新输入。");
                }
            }
            Err(_) => {
                //println!("无效的端口号，请重新输入。");
            }
        }
    }
	print!("\n添加节点的别名前缀(方便自己辨认)，默认不添加：");
    let prefix: String = input("");
    let result = build_vless_nodes(context_list, selected_port, &prefix, &v_sni, &v_host, &v_uuid,&tls_mode);

    let result_str = result.join("\n");

    fs::write("output.txt", result_str).expect("Failed to write to file");
    println!("节点已写入到output.txt文件中!\n");
	println!("程序执行完成,3秒后关闭窗口...");
	thread::sleep(Duration::from_secs(3));
}


fn input(prompt: &str) -> String {
  print!("{}", prompt);
  io::stdout().flush().unwrap();

  let mut input = String::new();
  io::stdin().read_line(&mut input).expect("Failed to read line");

  input.trim().to_string()
}