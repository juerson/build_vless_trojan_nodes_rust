这是Windows系统下的CLI程序工具，主要用于构建cf节点的分享链接，支持的协议有`vless+ws[+tls]`、`trojan+ws[+tls]`、`shadowsocks+v2ray-plugin+websocket[+tls]`。

## CLI命令：

执行命令：`build_cfwks_nodes.exe -h`


```
Usage: build_cfwks_nodes [OPTIONS]

Options:
  -f <config_path>        配置文件 [default: config.yaml]
  -i <data_path>          输入的数据文件(数据是ip、域名的txt/csv文件) [default: result.csv]
  -o <output_path>        输出的结果文件 [default: output.txt]
  -n <count>              从数据文件中，最大读取数 [default: 300]
  -s <selected_type>      选择指定的代理类型，可选值：vless,trojan,ss [default: ]
  -e <excluded_type>      排除不要的代理类型，可选值：vless,trojan,ss [default: ]
  -c <column_name>        如果是csv数据文件，采用哪列数据作为节点别名的一部分，可选值：colo,loc,region,city [default: colo]
      --tls [<tls>]       选择哪个TLS模式，只添加该参数不带值则是true值。可选值：none,true,false [possible values: none, true, false]
  -h, --help              Print help
  -V, --version           Print version
```

1、修改数据文件：`build_cfwks_nodes.exe -i=ip.txt`

2、只生成vless的节点：`build_cfwks_nodes.exe -s=vless`

3、排除ss的节点，也就是生成vless、trojan节点：`build_cfwks_nodes.exe -e=ss`

4、只使用非TLS端口的配置生成节点：`build_cfwks_nodes.exe --tls=false`

5、只使用TLS端口的配置生成节点：`build_cfwks_nodes.exe --tls=true`

## 使用

1、根据`config.yaml`文件的配置修改。

2、准备`result.csv`数据文件（或者自己通过命令行修改指定文件，支持`*.txt`和`*.csv`格式的文件）

3、运行`build_cfwks_nodes.exe`，结果输出到`output.txt`，将它们辅助到`Nekoray`中使用，如下：

<img src="images\图1.png" />

**注意：**

1、v2rayN中，不支持添加`v2ray-plugin`的`shadowsocks`分享链接。

2、从txt或csv文件读取到的端口，跟tls模式的端口冲突，会强行修改端口的。

