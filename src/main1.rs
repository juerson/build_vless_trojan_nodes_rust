use std::io;
use std::io::Write;

fn main() {
	println!("{}", "*".repeat(100));
	println!("CF VLESS链接中的端口，可以根据下面的TLS加密情况设置：");
	println!("非TLS加密：80 8080 8880 2052 2082 2086 2095");
	println!("是TLS加密：443 2053 2083 2087 2096 8443");
    println!("{}", "+".repeat(100));

	let mut link:String;
    loop {
		link = String::new();
        print!("输入VLESS LINK："); // 输入一个可用的vless节点
		io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut link).expect("读取输入失败");

        if link.starts_with("vless") && link.len() > 100 {
            break;
        }
    }

    println!("{}", "+".repeat(100));

    let before_string = link.chars().take(link.find('@').unwrap_or(link.len()) + 1).collect::<String>();
    let parameter_string = link
							.split('?')
							.nth(1)
							.and_then(|substring| substring.split('#').next())
							.unwrap_or("");

    loop {
        let mut addr = String::new();
        print!("输入优选的CDN地址(Address)：");
		io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut addr).expect("读取输入失败");
        let addr = addr.trim();

        if addr.is_empty() {
            continue;
        }

        let mut port = String::new();
		loop {
			print!("输入CDN端口(Port)：");
			io::stdout().flush().unwrap();
			std::io::stdin().read_line(&mut port).expect("读取输入失败");
			port = port.trim().to_string(); // 修剪端口字符串

			if port.chars().all(char::is_numeric) && port.trim().len() >= 2 {
				break; // 如果端口是两位以上的数字，跳出循环
			} else if port.is_empty() && parameter_string.contains("security=tls") {
				port = "2053".to_string();
				break;
			} else if port.is_empty() && !parameter_string.contains("security=tls") {
				port = "2052".to_string();
				break;
			} else {
				port.clear(); // 清空端口变量，以便重新输入
			}
		}

        let address = format!("{}:{}?", addr, port);
        let remarks = format!("#{}:{}", addr, port);
        let new_link = format!("{}{}{}{}", before_string, address, parameter_string, remarks);
		
		println!("复制下面的链接使用即可");
        println!("{}", "-".repeat(100));
        println!("{}", new_link);
        println!("{}", "-".repeat(100));
    }
}
