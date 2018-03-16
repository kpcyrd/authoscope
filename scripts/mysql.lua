descr = "local mysql"

function verify(user, password)
    return mysql_connect("127.0.0.1", 3306, user, password)
end
