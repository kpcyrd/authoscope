descr = "local mysql query"

function verify(user, password)
    sock = mysql_connect("127.0.0.1", 3306, "root", "my-secret-pw")
    if last_err() then return end

    rows = mysql_query(sock, 'SELECT VERSION(), :foo as foo', {
        foo='magic'
    })
    if last_err() then return end

    if rows[1] then
        print(rows[1])
        return true
    else
        return false
    end
end
