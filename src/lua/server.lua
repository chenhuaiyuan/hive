local server = { _is_ipv4 = true }

---绑定ip和端口
---@param addr string
---@param port number
---@return table
function server:bind(addr, port)
  self._addr = addr
  self._port = port
  return self
end

---是否是ipv6，调用此函数表示是ipv6
---@return table
function server:ipv6()
  self._is_ipv4 = false
  return self
end

---自定义异常处理函数
---@param exception function
---@return table
function server:exception(exception)
  self._exception = exception
  return self
end

---入口函数
---@param service function
---@return table
function server:serve(service)
  self._serve = service
  return self
end

function server:run()
  return {
    ['addr'] = self._addr,
    ['port'] = self._port,
    ['exception'] = self._exception,
    ['serve'] = self._serve,
    ['is_ipv4'] = self._is_ipv4
  }
end

return server
