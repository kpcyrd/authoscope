descr = "http"

function verify(user, password)
    session = http_mksession()

    -- set cookies
    req = http_request(session, 'GET', 'https://httpbin.org/anything', {
        user_agent="some-agent/0.1",
    })
    resp = http_send(req)
    if last_err() then return end

    json = json_decode(resp['text'])
    if last_err() then return end
    print(json)

    return false
end
