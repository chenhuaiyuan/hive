local server = {
  _is_ipv4 = true,
  _addr = '127.0.0.1',
  _port = 3000,
  _exception = nil,
  _serve = nil,
  _router = nil
}

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

---路由
---@param router userdata
---@return table
function server:router(router)
  self._router = router
  return self
end

function server:run()
  return {
    ['addr'] = self._addr,
    ['port'] = self._port,
    ['exception'] = self._exception,
    ['serve'] = self._serve,
    ['is_ipv4'] = self._is_ipv4,
    ['router'] = self._router
  }
end

return server
