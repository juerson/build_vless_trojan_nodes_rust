【rust语言编写的】批量构建 cf workers 的 vless 节点，提供IP或域名到ip.txt文件，运行[exe程序](https://github.com/juerson/build_vless_nodes_rust/releases/download/1.0/build_vless_nodes-x86_64-pc-windows-msvc.exe)按提示操作即可。

## 1、配置config.json文件

<img src="images/config的配置信息.png" />

**注意：**填写host值，如果使用`*.workers.dev`的域名，程序默认生成的vless链接没有tls加密的；添加非`workers.dev`后缀的域名，才能生成有tls加密信息的链接。

## 2、程序运行效果截图

<img src="images\程序截图.png" />

<img src="images\001.png" />



- **我不是程序员，需要哪些文件即可？**

<img src="images\正式使用需要这些文件即可.png" />

config.json文件（添加填写格式如下）：

```json
{
  "userID": "d342d11e-d424-4583-b36e-524ab1f0afa4",
  "host": "github.com",
  "sni": "github.com"
}
```

ip.txt文件可以自己创建，output.txt文件为程序最终生成的结果存放到这里。

**p.s. 使用该程序生成大量的vless节点，再将vless节点导入v2rayN等软件测试，有用的节点保留，没用的删除。**

