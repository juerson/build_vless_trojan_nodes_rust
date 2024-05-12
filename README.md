批量构建 cloudflare Workers/Pages 的 vless/trojan 节点，提供IP地址/域名、端口(可以省略端口，在exe程序操作设置默认端口)，就可以实现一条 vless/trojan 链接，扩展无数条 vless/trojan 链接。

### 1、修改配置config.json文件的信息，打开自己的v2rayN程序找

<img src="images/config的配置信息.png" />

`config.json`文件（填写格式如下，一定要根据自己的修改，没有值的就填""）：

```json
{
  "vless": {
    "userid": "填写自己的uuid",
    "host": "你的子域.pages.dev",
    "sni": "你的子域.pages.dev",
    "path": "/?ed=2048"
  },
  "trojan": {
    "password": "填写自己的密码",
    "host": "你的子域.pages.dev",
    "sni": "你的子域.pages.dev",
    "path": "/"
  }
}
```

注意：填写host值，如果使用 `*.workers.dev` 的域名，程序默认生成的 vless/trojan 链接没有tls加密的；添加非 `workers.dev` 后缀的域名，才能生成有 tls 加密信息的链接。

#### 其它文件说明：

（1）**ip.txt文件**：存放待写入 vless/trojan 节点的 IP 或域名，一行写一个，该文件可以自己创建；

（2）**output.txt文件**：程序运行后的结果存放到这里，文件不用自己创建，自动生成的。

### 2、程序运行支持的数据格式

也就是，在`ip.txt`文件中写什么样的数据，程序运行才能用？

<img src="images\数据格式.png" />

### 3、程序运行效果截图(含4个exe程序)

<img src="images\1.批量生成vless链接.png" />

<img src="images\2.批量生成trojan链接.png" />

<img src="images\3.使用链接，逐条生成版.png" />

<img src="images\4.使用链接，批量生成版.png" />

**p.s. 使用程序生成大量的 vless/trojan 节点，复制到 V2rayN 等软件测试，有用的节点保留，没用的删除。**
