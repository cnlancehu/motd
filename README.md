## Motd
**一款基于 Cli 的 Minecraft 服务器 Motd 查询工具**

### 有啥优点?
- 纯 Rust 编写，性能好 (好像也没啥用)
- 无需指定是基岩版还是 Java 版，基于协议自动识别
- 无需运行时，开箱即用
- Java 服务器图标显示

或者说体积小也算一个? 2mb 左右貌似也算小了，我不想使用 upx 等方法压缩程序，因为这样会在启动时浪费性能，快速的响应不是更好嘛

### 下载
| Windows | Linux | MacOS |
| :------------: | :------------: | :------------: |
| [x64](https://api.lance.fun/pkg/jump?id=motd&os=windows&arch=x86_64&version=latest&download=zip) | [x64](https://api.lance.fun/pkg/jump?id=motd&os=linux&arch=x86_64&version=latest&download=zip) | [Apple silicon](https://api.lance.fun/pkg/jump?id=motd&os=macos&arch=aarch64&version=latest&download=zip) |
| [x86](https://api.lance.fun/pkg/jump?id=motd&os=windows&arch=x86&version=latest&download=zip) | [x86](https://api.lance.fun/pkg/jump?id=motd&os=linux&arch=x86&version=latest&download=zip) | [Intel](https://api.lance.fun/pkg/jump?id=motd&os=macos&arch=x86_64&version=latest&download=zip) |
| [Arm64](https://api.lance.fun/pkg/jump?id=motd&os=windows&arch=aarch64&version=latest&download=zip) | [Arm64](https://api.lance.fun/pkg/jump?id=motd&os=linux&arch=aarch64&version=latest&download=zip) |


### 使用方法
```bash
motd <IP地址或域名> <端口 (为 19132 或 25565 时可省略)>
motd <IP地址或域名> <端口 (为 19132 或 25565 时可省略)>
```
例如
```bash
motd zqat.top
motd zqat.top 25565
motd zqat.top:25565
```

### 屏幕截图
![截图](https://get.lance.fun/ops/motd/sc/1.png)