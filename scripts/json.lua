descr = "json"

function verify(user, password)
    x = json_encode({
        hello="world",
        almost_one=0.9999,
        list={1,3,3,7},
        data={
            user=user,
            password=password,
            empty=nil
        }
    })
    print(x)

    y = json_decode(x)
    print(y)
    print(y["data"]["user"])

    return true
end

