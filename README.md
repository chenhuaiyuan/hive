# Hive

尝试打造一个rust + lua的 web framework。
后期尝试结合更多语言

## 功能

- [ ] http2.0支持
- [x] lua dev 模式下自动热更新
- [ ] lua release 模式下重载
- [x] websocket(可以使用，还有需要优化的地方)
- [ ] js支持（使用v8引擎）

## feature

默认只开启lua特性

| feature       | 功能                | 开启方法:                       |
| ------------- | ------------------ | ------------------------------ |
| lua_hotfix    | 开启dev模式下热更新   | --features "lua lua_hotfix"    |
| ws            | 开启websocket功能    | --features "lua ws"            |
| mysql         | 开启mysql功能        | --features "lua mysql"         |
| h2            | 开启http2功能(未完成) | --features "lua h2"            |
| create_object | 允许使用--create命令  | --features "lua create_object" |
| hive_log      | 开启log功能          | --features "lua hive_log"      |
| lua_file_data | 开启此功能可以实现上传和下载文件功能，如果不使用上传和下载功能，可以不用开启，使form表单提交速度更快 | --features "lua lua_file_data" |
| lua_json      | 开启json功能         | --features "lua lua_json"      |

## 自带库安装

目前只有lua库，放在ext目录下。
需要手动安装，安装方法：在终端进入对应的库目录，运行`luarocks make`。

### 准备开发的库

- [x] mysql(不打算使用此库，因为这个没有使用到异步库，性能不佳)
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

# js 此功能还未完成，请不要运行以下命令
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
