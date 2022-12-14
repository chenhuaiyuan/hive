local req = require "req"

local data = req.req():build():get("http://www.baidu.com/"):call():into_string()
print(data)
