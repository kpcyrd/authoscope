descr = "http post"

function verify(user, password)
    session = http_mksession()

    -- send login
    req = http_request(session, 'POST', 'https://httpbin.org/post', {
        json={
            user=user,
            password=password,
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp["status"] ~= 200 then return "invalid status code" end

    json = json_decode(resp['text'])
    if last_err() then return end
    print(json)

    return true
end
