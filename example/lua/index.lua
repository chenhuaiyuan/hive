_RESPONSE = require 'response'
local exception = dofile 'exception/base_exception.lua'
require 'config'
local query = loadfile 'orm/query.lua' -- 支持mysql和sqlite
-- local mongo = require 'mongo'
local router = require 'route'

-- query().new(MYSQL_USER, MYSQL_PASS, MYSQL_HOST, DATABASE)
-- query().open('./data/test.db3') -- 需要安装sqlite扩展， 扩展在ext目录下
-- local client = mongo.Client(MONGO)
-- Mongo = client:getDatabase(DATABASE)

-- dev_exec支持热更新
local function dev_exec(method, path, req)
  -- local remote_addr = req:remote_addr()
  -- local headers = req:headers()
  local params = { _request = req }
  local handler = router:execute(method, path)
  if handler.is_exist then
    params._router_params = handler.router_params
    if handler.middleware ~= nil then
      local is_pass = false;
      is_pass, params._user_info = handler.middleware(req)
      if not is_pass then
        local res = { code = 5001, message = 'Failed to verify token', data = '' }
        return hive.response.new():status(200):headers({
          ['Content-type'] = 'application/json'
        }):body(res)
      end
    end
    return handler.func(params)
  else
    return hive.response.new():status(404):headers({
      ['Content-type'] = 'application/json'
    }):body({
      ['code'] = 404,
      ['data'] = '',
      ['message'] = 'Not Found'
    })
  end
end

-- 不支持热更新，但速度更快
local function execute(is_exist, func, middleware, req, router_params)
  local params = { _request = req, _router_params = router_params }
  if is_exist then
    if middleware ~= nil then
      local is_pass = false
      is_pass, params._user_info = middleware(req)
      if not is_pass then
        local res = { code = 5001, message = 'Failed to verify token', data = '' }
        return hive.response.new():status(200):headers({
          ['Content-type'] = 'application/json'
        }):body(res)
      end
    end
    return func(params)
  else
    return hive.response.new():status(404):headers({
      ['Content-type'] = 'application/json'
    }):body({
      ['code'] = 404,
      ['data'] = '',
      ['message'] = 'Not Found'
    })
  end
end

-- 如果没有开启lua_hotfix特性，使用这个
local s = hive.server():bind("127.0.0.1", 3000):router(router:raw()):exception(exception):serve(execute)
-- 如果开启lua_hotfix特性，使用这个
-- local s = hive.server():bind("127.0.0.1", 3000):exception(exception):serve(dev_exec) -- 开发环境下使用这个，支持自动热更新
return s:run()
