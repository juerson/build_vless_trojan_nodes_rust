use regex::Regex;
use std::borrow::Cow;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use indexmap::IndexSet;
use ipnetwork::IpNetwork;

fn main() {
    println!("本工具的功能：将TXT文件中数据，组成新的vless链接，实现一条vless链接，扩展无数条vless链接。\n");
    let filename = "ip.txt";
    if !is_file_existing_and_non_empty(filename) {
        println!("文件{}不存在或者文件为空！", filename);
        println!("{}", "+".repeat(50));
        println!("本程序支持的内容格式如下：");
        println!("192.168.1.1");
        println!("192.168.1.0/24");
        println!("192.168.1.1 443");
        println!("192.168.1.1,443");
        println!("192.168.1.1,443,字段a,字段b,字段c,...");
        println!("{}", "+".repeat(50));
        wait_for_enter();
        std::process::exit(1);
    }
    /* 获取用户输入的原始链接 */
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
    println!("\n下面输入默认端口，用于读取文件内容时，不存在端口情况下，添加这个端口。");
    /* 默认的端口 */
    let default_port = get_port_form_user_input();
    /* 读取文件中的IP/域名和PORT */
    let ip_with_port_vec = read_words_from_file(filename, default_port.as_str());
    // 用于存储多条OK的链接节点
    let mut new_link_vec = Vec::new();
    for item in ip_with_port_vec {
        let addr = item[0].to_string();
        let port = item[1].to_string();
        // 别名
        let remarks = if addr.contains(":") {
            format!("#[{}]", addr)
        } else {
            format!("#{}", addr)
        };
        // 地址
        let address = if addr.contains(":") {
            format!("[{}]:{}", addr, port)
        } else {
            format!("{}:{}", addr, port)
        };
        // 拼接成新的vless节点
        let new_link = format!(
            "{}{}{}{}",
            before_string, address, parameter_string, remarks
        );
        new_link_vec.push(new_link.to_string());
    }
    /* 写入文件 */
    let file_name = "output.txt";
    write_to_file(new_link_vec, file_name);

    println!("\n生成的vless链接已经写入{}文件中！", file_name);

    wait_for_enter();
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

fn get_port_form_user_input() -> String {
    let mut input_port = String::new();
    loop {
        print!("请在这里输入默认端口：");
        io::stdout().flush().unwrap();
        input_port.clear(); // 清除输入缓冲区的内容，防止多次输入，输入正确的input_port无法跳出循环
        std::io::stdin()
            .read_line(&mut input_port)
            .expect("读取输入失败");

        // 判断输入的内容是否全为数字，而且是2~5位的数字
        if input_port.trim().chars().all(char::is_numeric)
            && input_port.trim().len() >= 2
            && input_port.trim().len() <= 5
        {
            break;
        }
    }
    return input_port.trim().to_string();
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

/* 读取txt文件的内容，按行读取 */
fn read_words_from_file(filename: &str, default_port: &str) -> Vec<Vec<String>> {
    let mut result_set: IndexSet<Vec<String>> = IndexSet::new();
    if let Ok(file) = File::open(filename) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(ip_network) = line.trim().parse::<IpNetwork>() {
                    // 是 CIDR，生成 IP 地址
                    for ip in ip_network.iter() {
                        let mut parts = vec![ip.to_string()];
                        parts.push(default_port.to_string());
                        result_set.insert(parts);
                    }
                } else {
                    // 不是 CIDR，使用 split_line 进行处理
                    let parts = split_line(line, default_port);
                    result_set.insert(parts);
                }
            }
        }
    }
    result_set.into_iter().collect()
}

fn split_line(line: String, default: &str) -> Vec<String> {
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

fn write_to_file(data: Vec<String>, file_name: &str) {
    let mut file = match File::create(file_name) {
        Ok(f) => f,
        Err(_e) => return,
    };

    for line in data {
        match file.write_all(line.as_bytes()) {
            Ok(_) => (),
            Err(_e) => return,
        }

        match file.write_all(b"\n") {
            Ok(_) => (),
            Err(_e) => return,
        }
    }
}

fn wait_for_enter() {
    print!("按Enter键退出程序...");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
}
