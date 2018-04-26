descr = "local mysql"

function verify(user, password)
    mysql_connect("127.0.0.1", 3306, user, password)

    if last_err() then
        clear_err()
        return false
    else
        return true
    end
end
