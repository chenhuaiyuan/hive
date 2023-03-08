local response = {}

function response.json(data)
  return {
    ['status'] = 200,
    ['headers'] = {
      ['Content-type'] = 'application/json'
    },
    ['body'] = hive.to_json(data)
  }
end

function response.success(data, code, message)
  if nil == message then
    message = 'Ok'
  end

  if nil == code then
    code = 200
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
  return {
    ['status'] = 200,
    ['headers'] = {
      ['Content-type'] = 'text/html'
    },
    ['body'] = body
  }
end

return response
