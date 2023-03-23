local response = {}
local hive_response = hive.response

function response.json(data, is_object)
  local body
  if next(data) == nil then
    if is_object ~= nil then
      if is_object == false then
        body = hive.empty_array()
      end
    else
      body = hive.empty_array()
    end
  else
    -- body = hive.to_json(data)
    body = data
  end
  return hive_response.new():status(200):headers({
    ['Content-type'] = 'application/json'
  }):body(body)
end

function response.success(data, code, message)
  if nil == message then
    message = 'Ok'
  end

  if nil == code then
    code = 200
  end

  if type(data) == 'table' then
    if data.data ~= nil then
      if type(data.data) == 'table' and next(data.data) == nil then
        data.data = hive.empty_array()
      end
    elseif next(data) == nil then
      data = hive.empty_array()
    end
  end

  local resp = { code = code, message = message, data = data }
  return response.json(resp)
end

function response.fail(code, message, data)
  if nil == data then
    data = ''
  end
  local resp = { code = code, message = message, data = data }
  return response.json(resp)
end

function response.html(body)
  return hive_response.new():headers({
    ['Content-type'] = 'text/html'
  }):status(200):body(body)
end

function response.pay_success()
  return hive_response.new():status(200):headers({
    ['Content-type'] = 'text/plain'
  }):body('success')
  -- return {
  --   ['status'] = 200,
  --   ['headers'] = {
  --     ['Content-type'] = 'text/plain'
  --   },
  --   ['body'] = 'success'
  -- }
end

function response.pay_fail()
  return hive_response.new():status(200):headers({
    ['Content-type'] = 'text/plain'
  }):body('')
  -- return {
  --   ['status'] = 200,
  --   ['headers'] = {
  --     ['Content-type'] = 'text/plain'
  --   },
  --   ['body'] = ''
  -- }
end

return response
