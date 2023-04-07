local _M = {}
local valid = require 'utils.validation'
-- local tera = require 'utils.tera'
-- local orm = loadfile 'orm/query.lua'
local hive_response = hive.response

function _M.index(request)
  return {
    ['name'] = 'test',
    ['age'] = 21
  }
end

-- sqlite需要安装指定的库，后期可能会考虑到性能问题直接合并到hive程序中
-- function _M.sql_test()
--   local mysql = orm().mysql():db('test'):find()
--   local sqlite = orm().sqlite():db('test'):find('id', 'name') -- sqlite 需要给出对应的字段
-- end

function _M.get_user_info(request)
  local params = request._request:params()
  valid.require(params, { 'username', 'age' })
  valid.number(params, { 'age' })
  local user = { username = params.username, age = params.age }
  return _RESPONSE.success(user)
end

function _M.test(request)
  return "hello world"
end

-- 使用tera必须要先安装tera库，tera库在hive源代码的ext中
-- function _M.template(request)
--   return _RESPONSE.html(tera:view('test.html', { context = 'hello world' }))
-- end

-- websocket，需要开启特定功能才能使用
function _M.ws(request)
  local func = function(sender_map, sender, msg)
    local m = msg:to_text()
    local resp;
    if m == '123' then
      resp = 'hello'
    else
      resp = 'world'
    end
    local message = hive.ws_message.text(resp)

    sender:send(message)
    -- sender_map:send_all(message) -- 给所有用户发送
  end
  return request._request:upgrade(func)
end

return _M
