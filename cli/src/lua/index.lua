router = require "router"
local routes = router.new()

function home(req)
  local params = req.params()

  return {
    ["status"] = 200,
    ["headers"] = {
      ["a"] = "aaa"
    },
    ["body"] = "hello world"
  }
end


routes:match("GET", "/", home)
