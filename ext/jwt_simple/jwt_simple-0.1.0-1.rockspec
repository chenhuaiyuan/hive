package = "jwt_simple"
version = "0.1.0-1"
source = {
   url = "git+https://github.com/chenhuaiyuan/hive.git",
   tag = "0.1.0"
}
description = {
   homepage = "*** please enter a project homepage ***",
   license = "*** please specify a license ***"
}
dependencies = {
   "lua >= 5.1",
   "luarocks-build-rust-mlua",
}
build = {
   type = "rust-mlua",
   modules = {
      "jwt_simple"
   }
}
