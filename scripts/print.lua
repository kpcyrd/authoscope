descr = "print"

function b64u(s)
    s = s:gsub('%=', '')
    s = s:gsub('%+', '-')
    s = s:gsub('%/', '_')
    return s
end

function verify(user, password)
    print("user=".. user ..", password=" .. password)
    -- this is buggy with hlua 0.4.1
    print({user, password})
    print(b64u('as=+/=+/=+/df'))
    print({
        data={
            user=user,
            password=password
        }
    })
    return true
end
