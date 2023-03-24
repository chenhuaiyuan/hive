local insert = table.insert
local type = type
local gsub = string.gsub
local rep = string.rep
local reverse = string.reverse
local concat = table.concat
local sub = string.sub
local find = string.find
-- local p = require 'utils.print_table'

local orm = {
  _type = 'mysql',
  _engine = nil,
  _database = '',
  _table = '',
  _params = {}
}

local _type = {
  ['mysql'] = 1,
  ['sqlite'] = 2
}

---数组合并
---@param t1 table
---@param t2 table
---@return table
local function array_merge(t1, t2)
  if t2 == nil then
    return t1
  end
  for _, v in ipairs(t2) do
    insert(t1, v)
  end
  return t1
end

function orm.new(user, pass, host, database)
  if database ~= nil then
    orm._database = database
    MYSQL = hive.mysql.new(user or MYSQL_USER, pass or MYSQL_PASS, host or MYSQL_HOST, database)
  else
    MYSQL = hive.mysql.new(user or MYSQL_USER, pass or MYSQL_PASS, host or MYSQL_HOST)
  end
  -- orm._type = 'mysql'
  return orm
end

function orm.open(path)
  local sqlite = require 'sqlite'
  SQLITE = sqlite.connect.open(path)
  -- orm._type = 'sqlite'
  return orm
end

function orm.open_in_memory()
  local sqlite = require 'sqlite'
  SQLITE = sqlite.connect.open_in_memory()
  -- orm._type = 'sqlite'
  return orm
end

---通过flag开启一个sqlite
---@param path string
---@param flags table sqlite.flags
function orm.open_with_flags(path, flags)
  local sqlite = require 'sqlite'
  SQLITE = sqlite.connect.open_with_flags(path, flags)
  -- orm._type = 'sqlite'
  return orm
end

---@param flags table
function orm.open_in_memory_with_flags(flags)
  local sqlite = require 'sqlite'
  SQLITE = sqlite.connect.open_in_memory_with_flags(flags)
  -- orm._type = 'sqlite'
  return orm
end

function orm.mysql()
  orm._type = _type.mysql
  return orm
end

function orm.sqlite()
  orm._type = _type.sqlite
  return orm
end

---数据库
---@param db string|nil
---@param table string
---@return table
function orm:db(table, db)
  if db ~= nil then
    self._database = db
  end
  self._table = table
  return self
end

---数据库字段
---@param ... any
---@return table
function orm:columns(...)
  local columns = { ... }
  for i, val in ipairs(columns) do
    local as_exist = find(val, ' as ')
    local bracket_exist = find(val, '%(')
    local dot_exist = find(val, '%.')
    if as_exist == nil and bracket_exist == nil and dot_exist == nil then
      columns[i] = '`' .. columns[i] .. '`'
    end
  end
  self._columns = columns
  return self
end

function orm:raw_columns(...)
  self._columns = { ... }
  return self
end

---数据库条件
---@param key string
---@param operator any
---@param val any|nil
---@return table
function orm:where(key, operator, val)
  if self._wheres == nil then
    if val ~= nil then
      self._wheres = ' WHERE ' .. key .. operator .. ' ? '
      insert(self._params, val)
    else
      self._wheres = ' WHERE ' .. key .. ' = ? '
      insert(self._params, operator)
    end
  else
    if val ~= nil then
      self._wheres = self._wheres .. ' AND ' .. key .. operator .. ' ? '
      insert(self._params, val)
    else
      self._wheres = self._wheres .. ' AND ' .. key .. ' = ? '
      insert(self._params, operator)
    end
  end
  return self
end

---条件或
---@param key string
---@param operator any
---@param val any|nil
---@return table
function orm:or_where(key, operator, val)
  if self._wheres == nil then
    if val ~= nil then
      self._wheres = ' WHERE ' .. key .. operator .. ' ? '
      insert(self._params, val)
    else
      self._wheres = ' WHERE ' .. key .. ' = ? '
      insert(self._params, operator)
    end
  else
    if val ~= nil then
      self._wheres = self._wheres .. ' OR ' .. key .. operator .. ' ? '
      insert(self._params, val)
    else
      self._wheres = self._wheres .. ' OR ' .. key .. ' = ? '
      insert(self._params, operator)
    end
  end
  return self
end

---where key between value1 and value2
---@param key string
---@param value table
function orm:where_between(key, value)
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' BETWEEN ? AND ? '
    array_merge(self._params, value)
  else
    self._wheres = self._wheres .. ' AND ' .. key .. ' BETWEEN ? AND ? '
    array_merge(self._params, value)
  end
  return self
end

---or key between value1 and value2
---@param key string
---@param value table
---@return self
function orm:or_where_between(key, value)
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' BETWEEN ? AND ? '
    array_merge(self._params, value)
  else
    self._wheres = self._wheres .. ' OR ' .. key .. ' BETWEEN ? AND ? '
    array_merge(self._params, value)
  end
  return self
end

---where in
---@param key string
---@param values table
---@return table
function orm:where_in(key, values)
  if type(values) ~= 'table' then
    return hive.web_error(3003, 'where_in function parameter must be table')
  end
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' IN ('
  else
    self._wheres = self._wheres .. ' AND ' .. key .. ' IN ('
  end
  self._params = array_merge(self._params, values)
  local len    = #values
  local expr   = rep(',?', len)
  expr         = gsub(expr, ',', ')', 1)
  expr         = reverse(expr)
  self._wheres = self._wheres .. expr
  return self
end

---or in
---@param key string
---@param values table
---@return table
function orm:or_where_in(key, values)
  if type(values) ~= 'table' then
    return hive.web_error(3003, 'where_in function parameter must be table')
  end
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' IN ('
  else
    self._wheres = self._wheres .. ' OR ' .. key .. ' IN ('
  end
  self._params = array_merge(self._params, values)
  local len    = #values
  local expr   = rep(',?', len)
  expr         = gsub(expr, ',', ')', 1)
  expr         = reverse(expr)
  self._wheres = self._wheres .. expr
  return self
end

---key not in
---@param key string
---@param values table
---@return table
function orm:where_not_in(key, values)
  if type(values) ~= 'table' then
    return hive.web_error(3003, 'where_in function parameter must be table')
  end
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' NOT IN ('
  else
    self._wheres = self._wheres .. ' AND ' .. key .. ' NOT IN ('
  end
  self._params = array_merge(self._params, values)
  local len    = #values
  local expr   = rep(',?', len)
  expr         = gsub(expr, ',', ')', 1)
  expr         = reverse(expr)
  self._wheres = self._wheres .. expr
  return self
end

---or key not in (key1, key2, key3)
---@param key string
---@param values table
---@return table
function orm:or_where_not_in(key, values)
  if type(values) ~= 'table' then
    return hive.web_error(3003, 'where_in function parameter must be table')
  end
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' NOT IN ('
  else
    self._wheres = self._wheres .. ' OR ' .. key .. ' NOT IN ('
  end
  self._params = array_merge(self._params, values)
  local len    = #values
  local expr   = rep(',?', len)
  expr         = gsub(expr, ',', ')', 1)
  expr         = reverse(expr)
  self._wheres = self._wheres .. expr
  return self
end

---key is null
---@param key string
---@return table
function orm:where_is_null(key)
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' IS NULL '
  else
    self._wheres = self._wheres .. ' AND ' .. key .. ' IS NULL '
  end
  return self
end

---key is not null
---@param key string
---@return table
function orm:where_is_not_null(key)
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' IS NOT NULL '
  else
    self._wheres = ' AND ' .. key .. ' IS NOT NULL '
  end
  return self
end

---or key is null
---@param key string
---@return table
function orm:or_where_is_null(key)
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' IS NULL '
  else
    self._wheres = self._wheres .. ' OR ' .. key .. ' IS NULL '
  end
  return self
end

---or key is not null
---@param key string
---@return table
function orm:or_where_is_not_null(key)
  if self._wheres == nil then
    self._wheres = ' WHERE ' .. key .. ' IS NOT NULL '
  else
    self._wheres = self._wheres .. ' OR ' .. key .. ' IS NOT NULL '
  end
  return self
end

---limit offset, count
---@param offset number
---@param count number | nil
---@return table
function orm:limit(offset, count)
  if self._type == _type.mysql then
    if nil ~= count then
      self._limit = ' LIMIT ' .. offset .. ', ' .. count .. ' '
    else
      self.limit = ' LIMIT ' .. offset .. ' '
    end
  elseif self._type == _type.sqlite then
    if nil ~= count then
      self._limit = ' LIMIT ' .. count .. ' OFFSET ' .. offset .. ' '
    else
      self.limit = ' LIMIT ' .. offset .. ' '
    end
  end
  return self
end

---排序
---@param field string
---@param sort string|nil
---@return table
function orm:order_by(field, sort)
  if nil == sort then
    self._order_by = ' ORDER BY ' .. field .. ' ASC '
  else
    self._order_by = ' ORDER BY ' .. field .. ' ' .. sort .. ' '
  end
  return self
end

---分组
---@param ... string
---@return table
function orm:group_by(...)
  self._group_by = ' GROUP BY ' .. concat({ ... }, ',')
  return self
end

---内联
---@param table2 string 表名
---@param table1_field string
---@param table2_field string
---@return table
function orm:inner_join(table2, table1_field, table2_field)
  self._join = ' INNER JOIN ' .. table2 .. ' ON ' .. table1_field .. ' = ' .. table2_field .. ' '
  return self
end

---左联
---@param table2 string 表名
---@param table1_field string
---@param table2_field string
---@return table
function orm:left_join(table2, table1_field, table2_field)
  if self._type == _type.mysql then
    self._join = ' LEFT JOIN ' .. table2 .. ' ON ' .. table1_field .. ' = ' .. table2_field .. ' '
  elseif self._type == _type.sqlite then
    self._join = ' LEFT OUTER JOIN ' .. table2 .. ' ON ' .. table1_field .. ' = ' .. table2_field .. ' '
  end
  return self
end

---右联
---@param table2 string 表名
---@param table1_field string
---@param table2_field string
---@return table
function orm:right_join(table2, table1_field, table2_field)
  self._join = ' RIGHT JOIN ' .. table2 .. ' ON ' .. table1_field .. ' = ' .. table2_field .. ' '
  return self
end

---交叉联表
---@param table2 string 表名
---@return table
function orm:cross_join(table2)
  self._join = ' CROSS JOIN ' .. table2 .. ' '
  return self
end

function orm:find(...)
  local columns = { ... }
  -- if self._columns ~= nil then
  --   columns = array_merge(self._columns, {...})
  -- else
  --   columns = {...}
  -- end
  local sql
  if self._columns ~= nil and next(self._columns) ~= nil then
    sql = 'SELECT ' .. concat(self._columns, ',')
  else
    sql = 'SELECT * '
  end

  if self._database ~= '' then
    if self._join == nil then
      sql = sql .. ' FROM `' .. self._database .. '`.' .. self._table .. ' '
    else
      sql = sql .. ' FROM ' .. self._database .. '.' .. self._table .. ' '
    end
  else
    sql = sql .. ' FROM ' .. self._table .. ' '
  end

  if self._join ~= nil then
    sql = sql .. self._join
  end

  if self._wheres ~= nil then
    sql = sql .. self._wheres
  end

  if self._group_by ~= nil then
    sql = sql .. self._group_by
  end

  if self._order_by ~= nil then
    sql = sql .. self._order_by
  end

  if self._limit ~= nil then
    sql = sql .. self._limit
  end
  local data
  if self._type == _type.mysql then
    data = MYSQL:exec_first(sql, self._params)
  elseif self._type == _type.sqlite then
    data = SQLITE:query_first(sql, self._params, columns)
  end
  return data
end

---执行原始sql
---@param sql string
---@param params table | nil
function orm:exec_first(sql, params)
  return MYSQL:exec_first(sql, params)
end

---执行原始sql
---@param sql string
---@param params table | nil
function orm:exec(sql, params)
  if self._type == 'mysql' then
    return MYSQL:exec(sql, params)
  elseif self._type == 'sqlite' then
    return SQLITE:execute(sql, params)
  end
end

---批量执行原始sql
---@param sql string
---@param params table | nil
function orm:exec_batch(sql, params)
  if self._type == 'mysql' then
    MYSQL:exec_batch(sql, params)
  elseif self._type == 'sqlite' then
    SQLITE:execute_batch(sql)
  end
end

function orm:find_all(...)
  local columns = { ... }
  -- if self._columns ~= nil then
  --   columns = array_merge(self._columns, {...})
  -- else
  --   columns = {...}
  -- end
  local sql
  if self._columns ~= nil and next(self._columns) ~= nil then
    sql = 'SELECT ' .. concat(self._columns, ',')
  else
    sql = 'SELECT * '
  end
  if self._database ~= '' then
    if self._join == nil then
      sql = sql .. ' FROM `' .. self._database .. '`.' .. self._table .. ' '
    else
      sql = sql .. ' FROM ' .. self._database .. '.' .. self._table .. ' '
    end
  else
    sql = sql .. ' FROM ' .. self._table .. ' '
  end

  if self._join ~= nil then
    sql = sql .. self._join
  end

  if self._wheres ~= nil then
    sql = sql .. self._wheres
  end

  if self._group_by ~= nil then
    sql = sql .. self._group_by
  end

  if self._order_by ~= nil then
    sql = sql .. self._order_by
  end

  if self._limit ~= nil then
    sql = sql .. self._limit
  end
  local data
  -- print(sql)
  if self._type == _type.mysql then
    data = MYSQL:exec(sql, self._params)
  elseif self._type == _type.sqlite then
    data = SQLITE:query(sql, self._params, columns)
  end
  return data
end

function orm:insert(data)
  local sql = 'INSERT INTO '
  local values = 'VALUES('
  local params = {}
  if self._database ~= '' then
    sql = sql .. '`' .. self._database .. '`.' .. self._table .. ' ('
  else
    sql = sql .. self._table .. ' ('
  end
  for key, val in pairs(data) do
    sql = sql .. '`' .. key .. '`,'
    if type(val) == 'string' and val:upper() == 'NULL' then
      values = values .. 'NULL,'
    else
      values = values .. '?,'
      insert(params, val)
    end
  end
  sql    = sub(sql, 1, -2) .. ')'
  values = sub(values, 1, -2) .. ')'
  if self._type == _type.mysql then
    return MYSQL:exec(sql .. values, params)
  elseif self._type == _type.sqlite then
    return SQLITE:insert(sql .. values, params)
  end
end

---批量插入
---@param fields table
---@param data table|nil
function orm:batch_insert(fields, data)
  local sql = 'INSERT INTO '
  local values = 'VALUES('
  if self._database ~= '' then
    sql = sql .. '`' .. self._database .. '`.' .. self._table .. ' ('
  else
    sql = sql .. self._table .. ' ('
  end
  local params = {}
  local is_table = false
  local is_no_table = false
  local once = 1
  for _, val in pairs(fields) do
    if type(val) == 'table' then
      if is_no_table then
        return hive.web_error(3002, '在批量插入数据时发现数据错误')
      end
      is_table = true
      local temp_params = {}
      for k, v in pairs(val) do
        if once == 1 then
          sql = sql .. '`' .. k .. '`,'
          values = values .. '?,'
        end
        insert(temp_params, v)
      end
      insert(params, temp_params)
      once = once + 1
    else
      if is_table then
        return hive.web_error(3001, '在批量插入数据时发现数据错误')
      else
        if is_no_table == false then
          sql = sql .. '`' .. val .. '`,'
        end
        is_no_table = true
      end
    end
  end
  if is_no_table == true and is_table == false then
    if data ~= nil then
      for i, v in ipairs(data) do
        values = values .. '?,'
        params[i] = v
      end
    end
  end
  sql = sub(sql, 1, -2)
  values = sub(values, 1, -2)
  sql = sql .. ')'
  values = values .. ')'
  MYSQL:exec_batch(sql .. values, params)
end

function orm:save(data)
  if self._wheres ~= nil then
    local res = self:find()
    if next(res) == nil then
      return self:insert(data)
    else
      return self:update(data)
    end
  else
    return self:insert(data)
  end
end

function orm:update(data)
  local sql
  if self._database ~= '' then
    sql = 'UPDATE `' .. self._database .. '`.' .. self._table .. ' SET '
  else
    sql = 'UPDATE ' .. self._table .. ' SET '
  end
  local params = {}
  for key, val in pairs(data) do
    if type(val) == 'string' and val:upper() == 'NULL' then
      sql = sql .. ' `' .. key .. '` = NULL,'
    elseif type(val) == 'table' then
      sql = sql .. ' `' .. key .. '` = '
      for _, v in ipairs(val) do
        sql = sql .. v
      end
      sql = sql .. ','
    else
      sql = sql .. ' `' .. key .. '` = ?,'
      insert(params, val)
    end
  end
  sql = sub(sql, 1, -2)
  if self._wheres ~= nil then
    sql = sql .. self._wheres
  end
  params = array_merge(params, self._params)
  if self._type == _type.mysql then
    return MYSQL:exec(sql, params)
  elseif self._type == _type.sqlite then
    return SQLITE:execute(sql, params)
  end
end

function orm:delete(datetime)
  local sql
  if SOFT_DELETE ~= nil and SOFT_DELETE then
    if self._database ~= '' then
      sql = 'UPDATE `' .. self._database .. '`.' .. self._table .. ' SET ' .. DELETEDTIME .. ' = ? '
    else
      sql = 'UPDATE ' .. self._table .. ' SET ' .. DELETEDTIME .. ' = ? '
    end
  else
    if self._database ~= '' then
      sql = 'DELETE FROM `' .. self._database .. '`.' .. self._table .. ' '
    else
      sql = 'DELETE FROM ' .. self._table .. ' '
    end
  end
  if datetime == nil then
    datetime = os.time()
  end
  local params = { datetime }
  if self._wheres ~= nil then
    sql = sql .. self._wheres
  end
  params = array_merge(params, self._params)
  if self._type == _type.mysql then
    return MYSQL:exec(sql, params)
  elseif self._type == _type.sqlite then
    return SQLITE:execute(sql, params)
  end
end

function orm:count()
  local sql
  if self._database ~= '' then
    sql = 'SELECT COUNT(*) as count FROM `' .. self._database .. '`.' .. self._table .. ' '
  else
    sql = 'SELECT COUNT(*) as count FROM ' .. self._table .. ' '
  end
  if self._join ~= nil then
    sql = sql .. self._join
  end

  if self._wheres ~= nil then
    sql = sql .. self._wheres
  end

  if self._group_by ~= nil then
    sql = sql .. self._group_by
  end

  if self._limit ~= nil then
    sql = sql .. self._limit
  end
  local data
  if self._type == _type.mysql then
    data = MYSQL:exec_first(sql, self._params)
  elseif self._type == _type.sqlite then
    data = SQLITE:query_first(sql, self._params, { 'count' })
  end
  return data.count or 0
end

return orm
