descr = "print"

function verify(user, password)
    print("user=".. user ..", password=" .. password)
    -- this is buggy with hlua 0.4.1
    print({user, password})
    print({
        data={
            user=user,
            password=password
        }
    })
    return true
end
