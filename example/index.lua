_RESPONSE = require 'response'
local exception = dofile 'exception/base_exception.lua'
require 'config'
-- local mongo = require 'mongo'

-- local mysql = loadfile 'orm/mysql.lua'
local router = require 'route'


-- mysql().new(MYSQL_USER, MYSQL_PASS, MYSQL_HOST)

-- local client = mongo.Client(MONGO)
-- Mongo = client:getDatabase(DATABASE)


local function exec(method, path, req)
  local remote_addr = req:remote_addr()
  local headers = req:headers()
  local bool, func, middleware, params = router:execute(method, path,
    { _request = req, _remote_addr = remote_addr, _headers = headers })
  if bool then
    if middleware then
      local is_pass = false;
      is_pass, params._user_info = middleware(req)
      if not is_pass then
        local res = { code = 5001, message = 'Failed to verify token', data = '' }
        return {
          ['status'] = 200,
          ['headers'] = {
            ['Content-type'] = 'application/json'
          },
          ['body'] = hive.table_to_json(res)
        }
      end
    end
    return func(params)
  else
    return {
      ['status'] = 404,
      ['headers'] = {
        ['Content-type'] = 'application/json'
      },
      ['body'] = hive.table_to_json({
        ['code'] = 404,
        ['data'] = '',
        ['message'] = 'not found'
      })
    }
  end
end

-- return exec, exception
local s = hive.server():bind("127.0.0.1", 3000):exception(exception):serve(exec)
return s:run()
