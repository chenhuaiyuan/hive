local t = require 'tera'

local _M = {}

_M._tera = t.tera.new('views/**/*')

---生成html
---@param html_name string
---@param context table
---@return string
function _M:view(html_name, context)
  if hive.env.dev then
    self._tera:full_reload()
  end
  return self._tera:render(html_name, context)
end

---添加模版文件
---@param file_path string
---@param rename string|nil
function _M:add_template_file(file_path, rename)
  self._tera:add_template_file(file_path, rename)
end

---添加模版
---@param name string
---@param context string
function _M:add_template(name, context)
  self._tera:add_raw_template(name, context)
end

---生成一次性html
---@param input string
---@param context table
---@param autoescape boolean
---@return unknown
function _M.one_off(input, context, autoescape)
  return t.tera.one_off(input, context, autoescape)
end

---html转义
---@param input string
---@return string
function _M.escape_html(input)
  return t.escape_html(input)
end

return _M
