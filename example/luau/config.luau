-- mongodb
-- MONGO_USER = 'root'
-- MONGO_PASS = 'root123'
-- MONGO = 'mongodb://' .. MONGO_USER .. ':' .. MONGO_PASS .. '@127.0.0.1:27017'

DATABASE = 'cena_warehouse'

TOKEN_PRIVATE_KEY = 'sdkfwiehfk13j2138sejdfr3q8fsjdfhw38fdmvn'

-- mysql
MYSQL_HOST = '127.0.0.1:3306'
MYSQL_USER = 'root'
MYSQL_PASS = 'root123'

-- 如果为true，当执行delete sql语句的时候会被update代替，会给DELETEDTIME字段插入数据
-- 如果为false，当执行delete sql语句时，将直接删除数据
SOFT_DELETE = true

-- airwallex
-- AIRWALLEX_API_KEY = '1727c6a46f21362b98b5f5dcd54df248773683cb1e3dc988ab624d422e3e4102a0df3e6c2fdb5521b1d33c7a0582e1b6'
-- AIRWALLEX_CLIENT_ID = 'Lc8DdP4DQ9K5tIsWour1EA'
-- test
-- AIRWALLEX_API_URL = 'https://api-demo.airwallex.com'
-- product
-- AIRWALLEX_API_URL = 'https://api.airwallex.com'


-- 变量值有：string timestamp table
-- string: xxxx-xx-xx xx:xx:xx
-- table: {year = xxxx, month = xx, day = xx, hour = xx, min = xx, sec = xx}
-- 默认值为timestamp
-- DATEFORMAT = 'timestamp'


CREATEDTIME = 'created_at'
UPDATEDTIME = 'updated_at'
DELETEDTIME = 'deleted_at'
