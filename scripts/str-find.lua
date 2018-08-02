descr = "str-find"

function verify(user, password)
    text = "You are currently logged in as 'foo', want to log out?"
    return text:find(user) ~= nil
end
