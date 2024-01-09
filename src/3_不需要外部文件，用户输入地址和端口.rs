use regex::Regex;
use std::io;
use std::io::Write;
use std::net::{Ipv4Addr, Ipv6Addr};
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;

fn main() {
    println!("本程序的用途：生成VLESS节点链接，只需要提供一条完整的VLESS链接，一个IP地址/域名以及端口，即可生成全新的vless链接。");

    println!("------------------------------------------------------------------------------------------------------------------------");
    println!("设置默认端口的建议：\n非TLS模式：80, 8080, 8880, 2052, 2082, 2086, 2095\n是TLS模式：443, 2053, 2083, 2087, 2096, 8443");
    println!("------------------------------------------------------------------------------------------------------------------------");
    // 获取用户输入的link链接
    let link = get_linke_from_user_input();
    /* 获取链接中，地址左边的字符串 */
    let before_string = link
        .chars()
        .take(link.find('@').unwrap_or(link.len()) + 1)
        .collect::<String>();
    // 获取链接中，端口右边的参数，即截取第一个“?”问号到第一个“#”之间的字符串（已经忽略这两个字符之间其它“?”问号）
    let parameter_string = link
        .find('?')
        .and_then(|start| link[start..].find('#').map(|end| &link[start..start + end]))
        .unwrap_or("");
    // 从vless链接中搜索security参数，如果是"security=tls"，说明使用tls加密模式，就大概知道端口可以设置什么？这里只是设置一个默认端口
    let mut default_port = if parameter_string.contains("security=tls") {
        "443".to_string()
    } else {
        "8080".to_string()
    };
    println!("------------------------------------------------------------------------------------------------------------------------");
    loop {
        // 获取用户输入的地址（IP、域名）
        let addr = get_host_from_user_input();
        // 获取用户输入的端口
        let input_port = get_port_form_user_input(default_port.clone());
        // 如果用户输入合法的端口，不为空，就使用用户输入的端口，否则使用前面设置的默认端口
        if !input_port.trim().is_empty() {
            default_port = input_port; // 将用户输入的端口赋值给default_port（也就是修改前面设置的默认端口）
        }
        let address = format!("{}:{}", addr, default_port);
        let remarks = format!("#{}", addr); // （不重要）设置vless节点的名称/别称
                                            // 重新组合一条新的vless链接（完整的vless链接）
        let new_link = format!(
            "{}{}{}{}",
            before_string, address, parameter_string, remarks
        );
        println!("- - - - - - - - - - - - - - - - - - - - - - - - - 生成的vless链接如下：- - - - - - - - - - - - - - - - - - - - - - - - -");
        println!("{}", new_link);
        // 复制到剪贴板
        let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
        clipboard.set_contents(new_link.to_owned()).unwrap();
        println!("\n节点的链接已复制到剪切板，可以黏贴到V2rayN、NekoBox等软件中使用！");
        println!("------------------------------------------------------------------------------------------------------------------------");
    }
}

/* 获取用户输入的vless链接 */
fn get_linke_from_user_input() -> String {
    loop {
        let mut link = String::new();
        print!("输入vless链接>> "); // 输入一个可用的vless节点
        io::stdout().flush().unwrap();
        link.clear(); // 清空字符串变量的内容
        std::io::stdin().read_line(&mut link).expect("读取输入失败");
        link = link.trim().to_string();
        // 判断输入的vless链接是否以"vless://"开头而且只有一个"vless://"，而且链接长度大于100
        if link.to_lowercase().starts_with(r"vless://")
            && Regex::new(r"vless://").unwrap().find_iter(&link).count() == 1
            && link.len() > 100
        {
            return link; // 跳出死循环
        } else {
            // 继续循环
        }
    }
}

/* 获取用户输入的IP地址/域名(也就是主机地址) */
fn get_host_from_user_input() -> String {
    loop {
        let mut addr = String::new();
        print!("输入IP地址/域名>> ");
        io::stdout().flush().unwrap();
        addr.clear(); // 清空字符串变量的内容
        io::stdin().read_line(&mut addr).expect("读取输入失败");
        // 清除字符串左右的空白
        addr = addr.trim().to_string();
        if is_valid_domain(&addr) {
            return addr.to_string();
        } else if is_valid_ipv4(&addr) {
            return addr.to_string();
        } else if is_valid_ipv6(&addr) {
            return format!("[{}]", addr.to_string());
        } else {
            // 继续循环
        }
    }
}

/* 获取用户输入的端口，用户不输入内容则使用default_port端口 */
fn get_port_form_user_input(default_port: String) -> String {
    loop {
        let mut port = String::new();
        print!("输入PORT端口[默认{}]>> ", default_port);
        io::stdout().flush().unwrap();
        port.clear(); // 清空字符串变量的内容
        std::io::stdin().read_line(&mut port).expect("读取输入失败");
        // 清除字符串左右的空白
        port = port.trim().to_string();
        if port.chars().all(char::is_numeric) && port.len() >= 2 && port.len() <= 5 {
            return port.to_string();
        } else if port.is_empty() {
            return "".to_string();
        } else {
            // 继续循环
        }
    }
}

/* 判断是否为域名 */
fn is_valid_domain(domain: &str) -> bool {
    let re =
        Regex::new(r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$").unwrap();
    re.is_match(domain)
}

/* 判断是否为IPv4地址 */
fn is_valid_ipv4(ip: &str) -> bool {
    match ip.parse::<Ipv4Addr>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

/* 判断是否为IPv6地址 */
fn is_valid_ipv6(ip: &str) -> bool {
    match ip.parse::<Ipv6Addr>() {
        Ok(_) => true,
        Err(_) => false,
    }
}
