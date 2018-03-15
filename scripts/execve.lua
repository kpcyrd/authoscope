descr = "exec test.sh"

function verify(user, password)
    return execve("./docs/test.sh", {user, password}) == 0
end
