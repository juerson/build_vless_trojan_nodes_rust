use std::io;
use std::io::Write;
use regex::Regex;
use std::net::{Ipv4Addr, Ipv6Addr};

fn main() {
	println!("{}", "*".repeat(100));
	println!("CF vless链接中的端口，可以根据下面所示的情况来设置：");
	println!("非TLS加密：80 8080 8880 2052 2082 2086 2095");
	println!("是TLS加密：443 2053 2083 2087 2096 8443");
    println!("{}", "+".repeat(100));
	
	// 获取用户输入的link链接
	let link = get_linke_from_user_input();

    println!("{}", "+".repeat(100));

    let before_string = link.chars().take(link.find('@').unwrap_or(link.len()) + 1).collect::<String>();
    let parameter_string = link.split('?').nth(1).and_then(|substring| substring.split('#').next()).unwrap_or("");

    loop {
        // 获取用户输入的地址（IP、域名）
		let addr = get_host_from_user_input();
		// 获取用户输入的端口
        let port = get_port_form_user_input(parameter_string);

        let address = format!("{}:{}?", addr, port);
        let remarks = format!("#{}:{}", addr, port);
        let new_link = format!("{}{}{}{}", before_string, address, parameter_string, remarks);
		
		println!("生成的vless链接：");
        println!("{}", "-".repeat(100));
        println!("{}", new_link);
        println!("{}", "-".repeat(100));
    }
}

fn get_linke_from_user_input() -> String {
    loop {
		let mut link = String::new();
        print!("输入vless链接："); // 输入一个可用的vless节点
		io::stdout().flush().unwrap();
		link.clear(); // 清空字符串变量的内容
        std::io::stdin().read_line(&mut link).expect("读取输入失败");
		link = link.trim().to_string();

        if link.starts_with("vless") && link.len() > 100 {
            return link
        }
    }
}

fn get_host_from_user_input() -> String {
    loop {
        let mut addr = String::new();
        print!("输入IP或域名：");
        io::stdout().flush().unwrap();
		addr.clear(); // 清空字符串变量的内容
        io::stdin().read_line(&mut addr).expect("读取输入失败");
		// 清除字符串左右的空白
        addr = addr.trim().to_string();
        if is_valid_domain(&addr) {
			return addr.to_string()
        } else if is_valid_ipv4(&addr) {
            return addr.to_string();
        } else if is_valid_ipv6(&addr) {
            return format!("[{}]",addr.to_string())
        }
    }
}

fn get_port_form_user_input(parameter_string:&str) -> String {
	loop {
		let mut port = String::new();
		print!("输入端口(PORT)：");
		io::stdout().flush().unwrap();
		port.clear(); // 清空字符串变量的内容
		std::io::stdin().read_line(&mut port).expect("读取输入失败");
		// 清除字符串左右的空白
		port = port.trim().to_string();

		if port.chars().all(char::is_numeric) && port.len() >= 2 && port.len() <= 5{
			return port.to_string()
		} else if port.is_empty() && parameter_string.contains("security=tls") {
			return "443".to_string()
		} else if port.is_empty() && !parameter_string.contains("security=tls") {
			return "8080".to_string()
		}
	}
}


fn is_valid_domain(domain: &str) -> bool {
    let re = Regex::new(r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$").unwrap();
    re.is_match(domain)
}

fn is_valid_ipv4(ip: &str) -> bool {
    match ip.parse::<Ipv4Addr>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn is_valid_ipv6(ip: &str) -> bool {
    match ip.parse::<Ipv6Addr>() {
        Ok(_) => true,
        Err(_) => false,
    }
}