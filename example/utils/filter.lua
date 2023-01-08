local _M = {}

function _M.empty_string(params)
  for i, v in pairs(params) do
    if type(v) == 'string' and v == '' then
      params[i] = nil
    end
  end
end

return _M
