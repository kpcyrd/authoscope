descr = "basic auth httpbin.org"

function verify(user, password)
    return http_basic_auth("https://httpbin.org/basic-auth/foo/buzz", user, password)
end
