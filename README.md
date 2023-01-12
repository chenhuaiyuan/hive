# Hive

`hive`打算成为一个支持多脚本语言的web framework。
目前只支持lua语言，未来将支持JavaScript和python

## features

- [x] lua dev 模式下自动热更新
- [ ] lua release 模式下热更新
- [x] websocket(可以使用，还有需要优化的地方)
- [ ] js支持（使用v8引擎）
- [ ] 允许js调用rust、c/c++等静态语言生成的动态库
- [ ] 允许调用node库

## 自带库安装

目前只有lua库，放在ext目录下。
需要手动安装，安装方法：在终端进入对应的库目录，运行`luarocks make`。

### 准备开发的库

- [x] mysql
- [ ] redis
- [x] tera(后端html模板引擎)
- [ ] grpc

### 安装

```bash
git clone https://github.com/chenhuaiyuan/hive.git
cd hive
# lua
# 使用lua运行时
cargo install --path . # 在本地安装hive软件，但默认不开启websocket功能，如需要使用websocket，请运行下面命令行
cargo install --path . --features "lua ws"

# js
# 使用v8运行时
bash download_macos_rusty_v8.sh
# 如果不是macos系统，可通过自己系统下载对应的librusty_v8.a，可以加快编译速度
# https://github.com/denoland/rusty_v8/releases
export RUSTY_V8_ARCHIVE=$HOME/.cache/rusty_v8/v0.61.0/librusty_v8_debug_x86_64-apple-darwin.a
# or
export RUSTY_V8_ARCHIVE=$HOME/.cache/rusty_v8/v0.61.0/librusty_v8_release_x86_64-apple-darwin.a
cargo install --path . --features "js"
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
