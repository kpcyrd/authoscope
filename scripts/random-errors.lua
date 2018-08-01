descr = "random errors"

function verify(user, password)
    if rand(0, 100) < 1 then
        return "random error"
    end

    return false
end
