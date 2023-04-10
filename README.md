# Hive

尝试打造一个rust + lua的 web framework。
后期尝试结合更多语言。
前期只注重功能实现。

## 功能

- [ ] http2.0支持
- [x] lua dev 模式下自动热更新
- [x] websocket(可以使用，还有需要优化的地方)

## feature

默认只开启lua54

| feature       | 功能                | 开启方法:                                    |
| ------------- | ------------------ | ------------------------------------------- |
| lua_hotfix    | 开启dev模式下热更新   | --features "lua_hotfix"    |
| ws            | 开启websocket功能    | --features "ws"            |
| mysql         | 开启mysql功能        | --features "mysql"         |
| h2            | 开启http2功能(未完成) | --features "h2"            |
| create_object | 允许使用--create命令  | --features "create_object" |
| hive_log      | 开启log功能          | --features "hive_log"      |
| lua_file_data | 开启此功能可以实现上传和下载文件功能，如果不使用上传和下载功能，可以不用开启，使form表单提交速度更快 | --features "lua_file_data" |
| luajit        | 用luajit代替lua      | --features "luajit" --no-default-features |
| luajit52      | 使用兼容lua52的luajit | --features "luajit52" --no-default-features |
| lua54         | 使用lua5.4，默认启用  | --features "lua54"          |
| lua53         | 使用lua5.3           | --features "lua53" --no-default-features |
| lua52         | 使用lua5.2           | --features "lua52" --no-default-features |
| lua51         | 使用lua5.1           | --features "lua51" --no-default-features |
| luau          | 使用luau             | --features "luau" --no-default-features  |

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
cargo install --path . # 默认只启用lua虚拟机
# 如果需要使用别的自带功能，可以通过--features
# 比如：
cargo install --path . --features "ws mysql hive_log"
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
