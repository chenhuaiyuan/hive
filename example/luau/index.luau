_RESPONSE = require 'response'
local exception = require('exception/base_exception')
require 'config'
local router = require 'route'

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
local function execute(is_exist: bool, func, middleware, req: userdata, router_params: table?)
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
