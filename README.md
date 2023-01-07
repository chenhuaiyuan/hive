# Hive

`hive`打算成为一个支持多脚本语言的web framework。
目前只支持lua语言，未来将支持JavaScript和python

## features

- [x] http/1.1
- [x] dev 模式下自动热更新
- [ ] release 模式下热更新
- [x] websocket(可以使用，还有需要优化的地方)

除此之外，hive还自带一些实用的库，有alipay，nanoid，req，xlsxwriter等库，存放在ext目录，需要手动安装，安装方法：在终端进入对应的库目录，运行`luarocks make`。

### 准备开发的库

- [x] mysql
- [ ] redis
- [x] tera(后端html模板引擎)
- [ ] grpc

### 安装

```bash
git clone https://github.com/chenhuaiyuan/hive.git
cd hive
cargo install --path . # 在本地安装hive软件，但默认不开启websocket功能，如需要使用websocket，请运行下面命令行
cargo install --path . --features "lua ws"
```

### 安装自带的库

```bash
cd hive/ext
cd mysql
luarocks make
```

### 命令行

创建项目

```bash
hive --create test # 创建test项目
cd test
cp config.lua.example config.lua
```

运行项目

```bash
hive
# or
hive -f index.lua
# or
hive --file index.lua
```

开启dev模式

```bash
hive -d
# or
hive --dev
```

更换监视目录，默认当前目录
此功能必须开启dev模式

```bash
hive -d -w controllers  # 只监视controllers目录，controllers目录下文件修改自动热更新
# or
hive -d --watch-dir controllers
```
