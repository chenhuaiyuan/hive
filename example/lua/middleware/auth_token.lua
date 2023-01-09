local jwt_simple = require 'jwt_simple'

local function auth_token(request)
  local jwt = jwt_simple.new(TOKEN_PRIVATE_KEY)
  local headers = request:headers()
  if headers.token then
    local is_pass, user = jwt:verify(headers.token)
    if is_pass then
      return true, user
    else
      return false, nil
    end
  else
    return false, nil
  end
end

return auth_token
