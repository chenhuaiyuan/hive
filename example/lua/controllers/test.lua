local _M = {}
local valid = require 'utils.validation'
local tera = require 'utils.tera'

function _M.index(request)
  return {
    ['status'] = 200,
    ['headers'] = {
      ['Content-type'] = 'text/html'
    },
    ['body'] = '<h1>hello world!</h1>'
  }
end

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
function _M.template(request)
  return tera:view('test.html', { context = 'hello world' })
end

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
