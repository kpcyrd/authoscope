descr = "smtp enum"

function verify(user, password)
    -- enumeration only, password is ignored
    sock = sock_connect("127.0.0.1", 25)
    if last_err() then return end
    sock_newline(sock, "\r\n")

    -- get the banner
    sock_recvline(sock)
    if last_err() then return end

    -- send ehlo
    sock_sendline(sock, "ehlo localhost")
    if last_err() then return end

    -- read extensions
    sock_recvline_regex(sock, "^250 ")
    if last_err() then return end

    -- start delivering an email
    sock_sendline(sock, "MAIL FROM:<root>")
    if last_err() then return end

    -- read reply
    sock_recvline(sock)
    if last_err() then return end
    -- TODO: verify starts with "250 "

    -- probe for user
    sock_sendline(sock, "RCPT TO:<" .. user .. ">")
    if last_err() then return end

    -- read reply
    reply = sock_recvn(sock, 1)
    if last_err() then return end

    -- check it was successful
    return reply == "2"
end
