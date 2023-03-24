-- local response = require 'response'
local response = hive.response

-- 对错误的处理
local function base_exception(code, message, status)
  local data = { code = code, message = message }
  local resp = response.new():headers({
    ['Content-type'] = 'application/json'
  })
  if status ~= nil then
    resp = resp:status(status)
  else
    resp = resp:status(200)
  end
  return resp:body(data)
  -- local resp = '<div>code: ' .. code .. '</div>'
  -- resp = resp .. '<div>message: ' .. message .. '</div>'
  -- return response.html(resp)
end

return base_exception
