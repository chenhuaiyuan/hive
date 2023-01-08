local response = require 'response'

-- 对错误的处理
local function base_exception(code, message)
  local resp = { code = code, message = message }
  return response.json(resp)
  -- local resp = '<div>code: ' .. code .. '</div>'
  -- resp = resp .. '<div>message: ' .. message .. '</div>'
  -- return response.html(resp)
end

return base_exception
