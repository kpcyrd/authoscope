descr = "binary"

function verify(user, password)
    print("\x00\xff")
    print(hex("\x00\xff"))

    return true
end
