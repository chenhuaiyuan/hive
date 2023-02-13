-- local mysql = require 'mysql'

local _mysql = {}
-- local p = require 'utils.print_table'

local OPERATOR = { "=", ">", "<", ">=", "<=", "!=", "LIKE", "NOT LIKE", "REGEXP", "NOT REGEXP", "RLIKE",
  "NOT RLIKE" }

---数组合并
---@param t1 table
---@param t2 table
---@return table
local function array_merge(t1, t2)
  for _, v in ipairs(t2) do
    table.insert(t1, v)
  end
  return t1
end

---判断是否包含
---@param oper string
---@return boolean
function OPERATOR.contain(oper)
  for _, v in pairs(OPERATOR) do
    if type(v) == 'string' and v == oper:upper() then
      return true
    end
  end
  return false
end

function _mysql.new(user, pass, host, database)
  if database ~= nil then
    _MYSQL = hive.mysql.new(user or MYSQL_USER, pass or MYSQL_PASS, host or MYSQL_HOST, database)
  else
    _MYSQL = hive.mysql.new(user or MYSQL_USER, pass or MYSQL_PASS, host or MYSQL_HOST)
  end
end

---数据库
---@param db string|nil
---@param table string
---@return table
function _mysql.db(table, db)
  -- print(_mysql._table)
  -- _mysql._columns = nil;
  -- _mysql._limit = nil;
  -- _mysql._order_by = nil;
  -- _mysql._wheres = nil;
  _mysql._database = db or DATABASE
  _mysql._table = table
  return _mysql
end

---数据库字段
---@param ... any
---@return table
function _mysql:columns(...)
  self._columns = { ... }
  return self
end

---数据库条件
---@param key string
---@param operator any
---@param val any|nil
---@return table
function _mysql:where(key, operator, val)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if self._wheres.data == nil then self._wheres.data = {} end
  if #(self._wheres.fields) == 0 then
    if val == nil then
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, '=')
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, operator)
    else
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, operator)
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, val)
    end
  else
    table.insert(self._wheres.fields, ' AND ')
    if val == nil then
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, '=')
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, operator)
    else
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, operator)
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, val)
    end
  end
  return self
end

---条件或
---@param key string
---@param operator any
---@param val any|nil
---@return table
function _mysql:or_where(key, operator, val)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if self._wheres.data == nil then self._wheres.data = {} end
  if #(self._wheres.fields) == 0 then
    if val == nil then
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, '=')
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, operator)
    else
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, operator)
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, val)
    end
  else
    table.insert(self._wheres.fields, ' OR ')
    if val == nil then
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, '=')
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, operator)
    else
      if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
        table.insert(self._wheres.fields, string.format(' %s ', key))
      else
        table.insert(self._wheres.fields, string.format(' `%s` ', key))
      end
      table.insert(self._wheres.fields, operator)
      table.insert(self._wheres.fields, '?')
      table.insert(self._wheres.data, val)
    end
  end
  return self
end

---where in
---@param key string
---@param values table
---@return table
function _mysql:where_in(key, values)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if self._wheres.data == nil then self._wheres.data = {} end
  if type(values) ~= 'table' then
    return hive.web_error(3003, 'where_in function parameter must be table')
  end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IN (', key))
    end

    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  else
    table.insert(self._wheres.fields, ' AND ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IN (', key))
    end
    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  end
  return self
end

---or in
---@param key string
---@param values table
---@return table
function _mysql:or_where_in(key, values)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if self._wheres.data == nil then self._wheres.data = {} end
  if type(values) ~= 'table' then
    return hive.web_error(3004, 'or_where_in function parameter must be table')
  end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IN (', key))
    end
    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  else
    table.insert(self._wheres.fields, ' OR ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IN (', key))
    end
    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  end
  return self
end

---key not in
---@param key string
---@param values table
---@return table
function _mysql:where_not_in(key, values)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if self._wheres.data == nil then self._wheres.data = {} end
  if type(values) ~= 'table' then
    return hive.web_error(3005, 'where_not_in function parameter must be table')
  end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s NOT IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` NOT IN (', key))
    end

    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  else
    table.insert(self._wheres.fields, ' AND ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s NOT IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` NOT IN (', key))
    end
    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  end
  return self
end

---or key not in (key1, key2, key3)
---@param key string
---@param values table
---@return table
function _mysql:or_where_not_in(key, values)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if self._wheres.data == nil then self._wheres.data = {} end
  if type(values) ~= 'table' then
    return hive.web_error(3006, 'or_where_not_in function parameter must be table')
  end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s NOT IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` NOT IN (', key))
    end
    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  else
    table.insert(self._wheres.fields, ' OR ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s NOT IN (', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` NOT IN (', key))
    end
    local len = #values
    for i, v in ipairs(values) do
      table.insert(self._wheres.fields, '?')
      if i < len then
        table.insert(self._wheres.fields, ',')
      end
      table.insert(self._wheres.data, v)
    end
    table.insert(self._wheres.fields, ') ')
  end
  return self
end

---key is null
---@param key string
---@return table
function _mysql:where_is_null(key)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NULL ', key))
    end

  else
    table.insert(self._wheres.fields, ' AND ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NULL ', key))
    end
  end
  return self
end

---key is not null
---@param key string
---@return table
function _mysql:where_is_not_null(key)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NOT NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NOT NULL ', key))
    end

  else
    table.insert(self._wheres.fields, ' AND ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NOT NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NOT NULL ', key))
    end
  end
  return self
end

---or key is null
---@param key string
---@return table
function _mysql:or_where_is_null(key)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NULL ', key))
    end
  else
    table.insert(self._wheres.fields, ' OR ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NULL ', key))
    end
  end
  return self
end

---or key is not null
---@param key string
---@return table
function _mysql:or_where_is_not_null(key)
  if self._wheres == nil then self._wheres = {} end
  if self._wheres.fields == nil then self._wheres.fields = {} end
  if #(self._wheres.fields) == 0 then
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NOT NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NOT NULL ', key))
    end
  else
    table.insert(self._wheres.fields, ' OR ')
    if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil then
      table.insert(self._wheres.fields, string.format(' %s IS NOT NULL ', key))
    else
      table.insert(self._wheres.fields, string.format(' `%s` IS NOT NULL ', key))
    end
  end
  return self
end

---limit offset, count
---@param offset number
---@param count number | nil
---@return table
function _mysql:limit(offset, count)
  if self._limit == nil then self._limit = {} end
  if nil ~= offset then
    self._limit.offset = offset
  end
  if nil ~= count then
    self._limit.count = count
  end
  return self
end

---排序
---@param field string
---@param sort string|nil
---@return table
function _mysql:order_by(field, sort)
  if self._order_by == nil then self._order_by = {} end
  if #(self._order_by) == 0 then
    if nil == sort then
      table.insert(self._order_by, field)
      table.insert(self._order_by, 'ASC')
    else
      table.insert(self._order_by, field)
      table.insert(self._order_by, sort:upper())
    end
  else
    table.insert(self._order_by, ',')
    if nil == sort then
      table.insert(self._order_by, field)
      table.insert(self._order_by, 'ASC')
    else
      table.insert(self._order_by, field)
      table.insert(self._order_by, sort:upper())
    end
  end
  return self
end

---分组
---@param ... string
---@return table
function _mysql:group_by(...)
  self._group_by = { ... }
  return self
end

---内联
---@param table2 string 表名
---@param table1_field string
---@param table2_field string
---@return table
function _mysql:inner_join(table2, table1_field, table2_field)
  if self._inner_join == nil then self._inner_join = {} end
  table.insert(self._inner_join, {
    ['table2'] = table2,
    ['table1_field'] = table1_field,
    ['table2_field'] = table2_field
  })
  return self
end

---左联
---@param table2 string 表名
---@param table1_field string
---@param table2_field string
---@return table
function _mysql:left_join(table2, table1_field, table2_field)
  if self._left_join == nil then self._left_join = {} end
  table.insert(self._left_join, {
    ['table2'] = table2,
    ['table1_field'] = table1_field,
    ['table2_field'] = table2_field
  })
  return self
end

---右联
---@param table2 string 表名
---@param table1_field string
---@param table2_field string
---@return table
function _mysql:right_join(table2, table1_field, table2_field)
  if self._right_join == nil then self._right_join = {} end
  table.insert(self._right_join, {
    ['table2'] = table2,
    ['table1_field'] = table1_field,
    ['table2_field'] = table2_field
  })
  return self
end

---交叉联表
---@param table2 string 表名
---@return table
function _mysql:cross_join(table2)
  if self._cross_join == nil then self._cross_join = {} end
  table.insert(self._cross_join, {
    ['table2'] = table2
  })
  return self
end

function _mysql:find()
  local sql = 'SELECT '
  if self._columns ~= nil then
    for _, v in ipairs(self._columns) do
      -- 带括号的处理，比如：count(*)
      local is_exist = string.find(v, '(', 1, true)

      -- 带有()的不做任何处理，比如：count(*)，sum(number)等等
      if is_exist ~= nil then
        sql = sql .. v .. ','
      else
        if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil or self._cross_join ~= nil then
          sql = sql .. string.format('%s,', v)
        else
          local is_found_as = string.find(v, 'as', 1, true)
          if is_found_as then
            sql = sql .. v .. ','
          else
            sql = sql .. string.format('`%s`,', v)
          end
        end
      end
    end
    sql = string.sub(sql, 1, -2)
  else
    sql = sql .. '*'
  end

  if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil or self._cross_join ~= nil then
    sql = sql .. string.format(' FROM `%s`.%s ', self._database, self._table)
    if self._inner_join ~= nil then
      for _, v in ipairs(self._inner_join) do
        sql = sql ..
            string.format('INNER JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
              v.table1_field,
              v.table2_field)
      end
    elseif self._left_join ~= nil then
      for _, v in ipairs(self._left_join) do
        sql = sql ..
            string.format('LEFT JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
              v.table1_field,
              v.table2_field)
      end
    elseif self._cross_join ~= nil then
      for _, v in ipairs(self._cross_join) do
        sql = sql .. string.format('CROSS JOIN `%s`.%s ', self._database, v.table2)
      end
    else
      for _, v in ipairs(self._right_join) do
        sql = sql ..
            string.format('RIGHT JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
              v.table1_field,
              v.table2_field)
      end
    end
  else
    sql = sql .. string.format(' FROM `%s`.`%s` ', self._database, self._table)
  end

  if self._wheres ~= nil then
    sql = sql .. ' WHERE '
    for _, v in ipairs(self._wheres.fields) do
      sql = sql .. v
    end
  end

  if self._wheres ~= nil and self._wheres.data ~= nil then
    local data = _MYSQL:exec_first(sql, self._wheres.data)
    -- for i, v in pairs(data) do
    --   if i == CREATEDTIME or i == UPDATEDTIME or i == DELETEDTIME then
    --     data[i] = tonumber(v)
    --   end
    -- end
    return data
  end
  local data = _MYSQL:exec_first(sql)
  -- for i, v in pairs(data) do
  --   if i == CREATEDTIME or i == UPDATEDTIME or i == DELETEDTIME then
  --     data[i] = tonumber(v)
  --   end
  -- end
  return data
end

---执行原始sql
---@param sql string
---@param params table | nil
function _mysql.exec_first(sql, params)
  return _MYSQL:exec_first(sql, params)
end

---执行原始sql
---@param sql string
---@param params table | nil
function _mysql.exec(sql, params)
  return _MYSQL:exec(sql, params)
end

function _mysql:find_all()
  local sql = 'SELECT '
  if self._columns ~= nil then
    for _, v in ipairs(self._columns) do
      local is_exist = string.find(v, '(', 1, true)

      -- 带有()的不做任何处理，比如：count(*)，sum(number)等等
      if is_exist ~= nil then
        sql = sql .. v .. ','
      else
        if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil or self._cross_join ~= nil then
          sql = sql .. string.format('%s,', v)
        else
          local is_found_as = string.find(v, 'as', 1, true)
          if is_found_as then
            sql = sql .. v .. ','
          else
            sql = sql .. string.format('`%s`,', v)
          end
        end
      end
    end
    sql = string.sub(sql, 1, -2)
  else
    sql = sql .. '*'
  end

  if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil or self._cross_join ~= nil then
    sql = sql .. string.format(' FROM `%s`.%s ', self._database, self._table)
    if self._inner_join ~= nil then
      for _, v in ipairs(self._inner_join) do
        sql = sql ..
            string.format('INNER JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
              v.table1_field,
              v.table2_field)
      end
    elseif self._left_join ~= nil then
      for _, v in ipairs(self._left_join) do
        sql = sql ..
            string.format('LEFT JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
              v.table1_field,
              v.table2_field)
      end
    elseif self._cross_join ~= nil then
      for _, v in ipairs(self._cross_join) do
        sql = sql .. string.format('CROSS JOIN `%s`.%s ', self._database, v.table2)
      end
    else
      for _, v in ipairs(self._right_join) do
        sql = sql ..
            string.format('RIGHT JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
              v.table1_field,
              v.table2_field)
      end
    end
  else
    sql = sql .. string.format(' FROM `%s`.`%s` ', self._database, self._table)
  end

  if self._wheres ~= nil then
    sql = sql .. ' WHERE '
    for _, v in ipairs(self._wheres.fields) do
      sql = sql .. v
    end
  end
  if self._group_by ~= nil then
    sql = sql .. ' GROUP BY '
    for _, v in ipairs(self._group_by) do
      sql = sql .. string.format(' `%s`,', v)
    end
    sql = string.sub(sql, 1, -2)
  end
  if self._order_by ~= nil then
    sql = sql .. ' ORDER BY '
    for _, v in ipairs(self._order_by) do
      sql = sql .. v .. ' '
    end
  end
  if self._limit ~= nil then
    if self._limit.count ~= nil then
      sql = sql .. string.format(' LIMIT %s, %s ', self._limit.offset, self._limit.count)
    else
      sql = sql .. ' LIMIT ' .. self._limit.offset
    end
  end

  if self._wheres ~= nil and self._wheres.data ~= nil then
    local data = _MYSQL:exec(sql, self._wheres.data)
    -- for i, v in ipairs(data) do
    --   if v[CREATEDTIME] ~= nil then data[i][CREATEDTIME] = tonumber(v[CREATEDTIME]) end
    --   if v[UPDATEDTIME] ~= nil then data[i][UPDATEDTIME] = tonumber(v[UPDATEDTIME]) end
    --   if v[DELETEDTIME] ~= nil and v[DELETEDTIME] ~= '' then data[i][DELETEDTIME] = tonumber(v[DELETEDTIME]) end
    -- end
    return data
  end
  local data = _MYSQL:exec(sql)
  -- for i, v in ipairs(data) do
  --   if v[CREATEDTIME] ~= nil then data[i][CREATEDTIME] = tonumber(v[CREATEDTIME]) end
  --   if v[UPDATEDTIME] ~= nil then data[i][UPDATEDTIME] = tonumber(v[UPDATEDTIME]) end
  --   if v[DELETEDTIME] ~= nil and v[DELETEDTIME] ~= '' then data[i][DELETEDTIME] = tonumber(v[DELETEDTIME]) end
  -- end
  return data
end

function _mysql:insert(data)
  local sql = 'INSERT INTO '
  local values = 'VALUES('
  sql = sql .. string.format('`%s`.`%s` (', self._database, self._table)
  local params = {}
  for key, val in pairs(data) do
    sql = sql .. string.format('`%s`,', key)
    if type(val) == 'string' and val:upper() == 'NULL' then
      values = values .. 'NULL,'
    else
      values = values .. '?,'
      table.insert(params, val)
    end
  end
  sql = string.sub(sql, 1, -2)
  values = string.sub(values, 1, -2)
  sql = sql .. ')'
  values = values .. ')'
  return _MYSQL:exec(sql .. values, params)
end

---批量插入
---@param fields table
---@param data table|nil
function _mysql:batch_insert(fields, data)
  local sql = 'INSERT INTO '
  local values = 'VALUES('
  sql = sql .. string.format('`%s`.`%s` (', self._database, self._table)
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
          sql = sql .. string.format('`%s`,', k)
          values = values .. '?,'
        end
        table.insert(temp_params, v)
      end
      table.insert(params, temp_params)
      once = once + 1
    else
      if is_table then
        return hive.web_error(3001, '在批量插入数据时发现数据错误')
      else
        if is_no_table == false then
          sql = sql .. string.format('`%s`,', val)
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
  sql = string.sub(sql, 1, -2)
  values = string.sub(values, 1, -2)
  sql = sql .. ')'
  values = values .. ')'
  _MYSQL:exec_batch(sql .. values, params)
end

local function length(t)
  local count = 0
  for _, _ in pairs(t) do
    count = count + 1
  end
  return count
end

function _mysql:save(data)
  if self._wheres ~= nil then
    local res = self:find()
    if length(res) == 0 then
      return self:insert(data)
    else
      return self:update(data)
    end
  else
    return self:insert(data)
  end
end

function _mysql:update(data)
  local sql = string.format('UPDATE `%s`.`%s` SET ', self._database, self._table)
  local params = {}
  for key, val in pairs(data) do
    if type(val) == 'string' and val:upper() == 'NULL' then
      sql = sql .. string.format(' `%s` = NULL,', key)
    elseif type(val) == 'table' then
      sql = sql .. string.format(' `%s` = ', key)
      for _, v in ipairs(val) do
        sql = sql .. v
      end
      sql = sql .. ','
    else
      sql = sql .. string.format(' `%s` = ?,', key)
      table.insert(params, val)
    end
  end
  sql = string.sub(sql, 1, -2)
  if self._wheres ~= nil then
    sql = sql .. ' WHERE '
    for _, v in ipairs(self._wheres.fields) do
      sql = sql .. v
    end
    params = array_merge(params, self._wheres.data)
  end
  return _MYSQL:exec(sql, params)
end

function _mysql:delete(datetime)
  local sql = string.format('UPDATE `%s`.`%s` SET deleted_at = ? ', self._database, self._table)
  if datetime == nil then
    datetime = os.time()
  end
  local params = { datetime }
  if self._wheres ~= nil then
    sql = sql .. ' WHERE '
    for _, v in ipairs(self._wheres.fields) do
      sql = sql .. v
    end
    params = array_merge(params, self._wheres.data)
  end
  return _MYSQL:exec(sql, params)
end

function _mysql:count()
  local sql = ''
  if self._inner_join ~= nil or self._left_join ~= nil or self._right_join ~= nil or self._cross_join ~= nil then
    sql = string.format('SELECT COUNT(*) as count FROM `%s`.%s ', self._database, self._table)
  else
    sql = string.format('SELECT COUNT(*) as count FROM `%s`.`%s` ', self._database, self._table)
  end

  if self._inner_join ~= nil then
    for _, v in ipairs(self._inner_join) do
      sql = sql ..
          string.format('INNER JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
            v.table1_field,
            v.table2_field)
    end
  elseif self._left_join ~= nil then
    for _, v in ipairs(self._left_join) do
      sql = sql ..
          string.format('LEFT JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
            v.table1_field,
            v.table2_field)
    end
  elseif self._right_join ~= nil then
    for _, v in ipairs(self._right_join) do
      sql = sql ..
          string.format('RIGHT JOIN `%s`.%s ON %s = %s ', self._database, v.table2,
            v.table1_field,
            v.table2_field)
    end
  elseif self._cross_join ~= nil then
    for _, v in ipairs(self._cross_join) do
      sql = sql .. string.format('CROSS JOIN `%s`.%s ', self._database, v.table2)
    end
  end

  local params = {}
  if self._wheres ~= nil then
    sql = sql .. ' WHERE '
    for _, v in ipairs(self._wheres.fields) do
      sql = sql .. v
    end
    if self._wheres.data ~= nil then
      params = array_merge(params, self._wheres.data)
    end
  end

  local sql_clone = ''
  if self._group_by ~= nil then
    sql = sql .. ' GROUP BY '
    for _, v in ipairs(self._group_by) do
      sql = sql .. string.format(' `%s`,', v)
    end
    sql = string.sub(sql, 1, -2)
    sql_clone = 'SELECT COUNT(*) as count FROM (' .. sql .. ') s'
  else
    sql_clone = sql
  end
  if #params == 0 then
    local data = _MYSQL:exec_first(sql_clone)
    return data.count or 0
  else
    local data = _MYSQL:exec_first(sql_clone, params)
    return data.count or 0
  end
end

return _mysql
