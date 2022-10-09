local r = require 'router'
router = r.new()

function home(req)
  local params = req.req:params()

  if (params["b"]) then
    print(params["b"])
  end
  print(params["a"])
  return {
    ["status"] = 200,
    ["headers"] = {
      ["a"] = "aaa"
    },
    ["body"] = "hello world"
  }
end


router:match("POST", "/", home)

function exec(method, path, ...)
  local req = ...
  local bool, resp = router:execute(method, path, {req = req})
  if (bool) then
    return resp
  else
    return {
      ["status"] = 404,
      ["body"] = "not found"
    }
  end
end

return exec
