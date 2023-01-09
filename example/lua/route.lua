-- local auth_token = require 'middleware.auth_token'
local test = require 'controllers.test'
local r = require 'router'
local router = r.new()


router:match('get', '/', test.index)
-- match第四个参数是传中间件函数
-- router:match('GET', '/test', test.index, auth_token)
router:match('get', '/get_user_info', test.get_user_info)
router:match('get', '/test', test.test)
router:match('get', '/template', test.template)
router:match('get', '/ws', test.ws)

return router
