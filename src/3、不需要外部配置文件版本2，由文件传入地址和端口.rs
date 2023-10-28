use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::io;
use std::fs;
use std::path::Path;

fn main() {
	println!("本工具的功能：将TXT文件中IP和PORT，写入到新vless链接的地址和端口中，实现一条vless链接，扩展无数条vless链接。\n");
	let filename = "ip.txt";
	if !is_file_existing_and_non_empty(filename){
		println!("文件{}不存在或者文件为空！",filename);
		println!("{}", "+".repeat(50));
		println!("文件{}中的内容格式如下：",filename);
		println!("{}", "-".repeat(50));
		println!("192.168.1.1 443\n192.168.1.2 8080\n192.168.1.3 80");
		println!("{}", "-".repeat(50));
		println!("或者：");
		println!("{}", "-".repeat(50));
		println!("192.168.1.1\n192.168.1.2\n192.168.1.3");
		println!("{}", "+".repeat(50));
		wait_for_enter();
        std::process::exit(1);
	}
	
	/* 获取用户输入的原始链接 */
	let mut link = String::new();
    loop {
        print!("输入原始VLESS链接：");
		io::stdout().flush().unwrap();
		link.clear(); // 清除输入缓冲区的内容，防止多次输入，输入正确的link无法跳出循环
        std::io::stdin().read_line(&mut link).expect("读取输入失败");
		// 粗略检查输入的链接是否为vless链接
        if link.starts_with("vless") && link.len() > 100 {
            break;
        }
    }
	let link_trim = link.trim();
	/* 获取链接中，地址左边的字符串、端口右边的参数 */
    let before_string = link_trim.chars().take(link_trim.find('@').unwrap_or(link_trim.len()) + 1).collect::<String>();
    let parameter_string = link_trim.split('?').nth(1).and_then(|substring| substring.split('#').next()).unwrap_or("");
	
	/* 默认的端口 */
	println!("\n下面输入默认端口，用于在TXT文件中没有端口时，添加这个输入的默认端口。");
	let mut input_port = String::new();
    loop {
        print!("请输入您要设置的默认端口：");
        io::stdout().flush().unwrap();
        input_port.clear(); // 清除输入缓冲区的内容，防止多次输入，输入正确的input_port无法跳出循环
        std::io::stdin().read_line(&mut input_port).expect("读取输入失败");

        // 判断输入的内容是否全为数字，而且是2~5位的数字
        if input_port.trim().chars().all(char::is_numeric) && input_port.trim().len() >= 2 && input_port.trim().len() <= 5 {
            break;
        }
    }
	// 移除输入的内容中，可能存在的左右空白
	let default_port = input_port.trim();
	
	/* 读取文件中的IP和PORT */
    let words = read_words_from_file(filename,default_port);
	// 用于存储多条OK的链接节点
	let mut new_link_vec = Vec::new();
	for item in words{
		let addr = item[0].to_string();
		let port = item[1].to_string();
		// 别名
		let remarks = if addr.contains(":") {
			format!("#[{}]:{}", addr, port)
		} else {
			format!("#{}:{}", addr, port)
		};
		// 地址
		let address = if addr.contains(":") {
			format!("[{}]:{}?", addr, port)
		} else {
			format!("{}:{}?", addr, port)
		};
        // 拼接成新的vless节点
        let new_link = format!("{}{}{}{}", before_string, address, parameter_string, remarks);
		new_link_vec.push(new_link.to_string());
	}
	/* 写入文件 */
	let file_name = "output.txt";
	write_to_file(new_link_vec,file_name);
	
	println!("\n生成的链接已经写入{}文件中！",file_name);
	
	wait_for_enter();
}

// ip.txt文件自检（文件是否存在、文件不为空）
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

fn read_words_from_file(filename: &str, default_port: &str) -> Vec<Vec<String>> {	
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut words = Vec::new();

    for line in reader.lines() {
		let line = line.unwrap().to_string();
		let word_vec = split_line(line,default_port);
		words.push(word_vec);
	}
    words
}

fn split_line(line: String, default: &str) -> Vec<String> {
	// 移除左右空白
	let line = line.trim().to_string();
	// 分割为单词
	let mut words: Vec<String> = line.split_whitespace().map(|w| w.to_string()).collect();

	// 如果只分割到1个单词,使用默认值作为第二个(端口)
	if words.len() == 1 {
		words.push(default.to_string());
	}
	words
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
    io::stdin().read_line(&mut input).expect("Failed to read line");
}