descr = "smtp"

function verify(user, password)
    -- TODO: enumeration only, password is ignored
    sock = sock_connect("127.0.0.1", 25)
    if last_err() then return end
    sock_newline(sock, "\r\n")

    x = sock_recvline(sock)
    if last_err() then return end

    sock_sendline(sock, "ehlo localhost")
    if last_err() then return end

    sock_recvline_regex(sock, "^250 ")
    if last_err() then return end

    sock_sendline(sock, "MAIL FROM:<root>")
    if last_err() then return end

    sock_recvline(sock)
    if last_err() then return end
    -- TODO: verify starts with "250 "

    sock_sendline(sock, "RCPT TO:<" .. user .. ">")
    if last_err() then return end

    reply = sock_recvn(sock, 1)
    if last_err() then return end

    return reply == "2"
end
