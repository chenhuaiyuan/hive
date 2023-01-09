local _M = {}

function _M.require(params, fields)
  for _, v in ipairs(fields) do
    if params[v] == nil then
      hive.web_error(2001, v .. '字段必填')
    end
  end
end

function _M.number(params, fields)
  for _, v in ipairs(fields) do
    if params[v] ~= nil then
      if type(params[v]) == 'string' and params[v] ~= '' then
        local num = tonumber(params[v])
        if num ~= nil then
          if type(num) ~= 'number' then
            hive.web_error(2002, v .. '字段必须是数字')
          else
            params[v] = num
          end
        else
          hive.web_error(2002, v .. '字段必须是数字')
        end
      else
        if params[v] ~= '' and type(params[v]) ~= 'number' then
          hive.web_error(2002, v .. '字段必须是数字')
        end
      end
    end
  end
end

function _M.string(params, fields)
  for _, v in ipairs(fields) do
    if params[v] ~= nil and type(params[v]) ~= 'string' then
      hive.web_error(2003, v .. '字段必须是字符串')
    end
  end
end

function _M.is_array(params)
  local _array = true
  for i, _ in pairs(params) do
    if type(i) ~= "number" then
      _array = false
      break
    end
  end
  return _array
end

function _M.array(params, fields)
  for _, v in ipairs(fields) do
    if params[v] ~= nil and type(params[v]) ~= 'table' then
      if _M.is_array(params[v]) == false then
        hive.web_error(2004, v .. '字段必须是数组')
      end
    end
  end
end

function _M.is_hash_map(params)
  local _map = true
  for i, _ in pairs(params) do
    if type(i) == 'number' then
      _map = false
      break
    end
  end
  return _map
end

function _M.is_multi_array(params)
  local _array = true
  for _, v in pairs(params) do
    if type(v) ~= "table" then
      _array = false
      break
    end
  end
  return _array
end

return _M
