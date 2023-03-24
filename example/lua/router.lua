local router = {}

function router.new()
  router.r = hive.router.new()
  return router
end

function router:match(method, path, func, middleware)
  self.r:match(method, path, func, middleware)
end

function router:execute(method, path)
  return self.r:execute(method, path)
end

function router:raw()
  return self.r
end

return router
