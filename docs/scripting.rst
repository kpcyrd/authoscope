Scripting
=========

A simple script could look like this:

.. code-block:: lua

    descr = "example.com"

    function verify(user, password)
        session = http_mksession()

        -- get csrf token
        req = http_request(session, 'GET', 'https://example.com/login', {})
        resp = http_send(req)
        if last_err() then return end

        -- parse token from html
        html = resp['text']
        csrf = html_select(html, 'input[name="csrf"]')
        token = csrf["attrs"]["value"]

        -- send login
        req = http_request(session, 'POST', 'https://example.com/login', {
            form={
                user=user,
                password=password,
                csrf=token
            }
        })
        resp = http_send(req)
        if last_err() then return end

        -- search response for successful login
        html = resp['text']
        return html:find('Login successful') ~= nil
    end

Please see the reference and [examples](/scripts) for all available functions.
Keep in mind that you can use `print(x)` and `badtouch oneshot` to debug your
script.

base64_decode
-------------

Decode a base64 string.

.. code-block:: lua

    base64_decode("ww==")

base64_encode
-------------

Encode a binary array with base64.

.. code-block:: lua

    base64_encode("\x00\xff")

clear_err
---------

Clear all recorded errors to prevent a requeue.

.. code-block:: lua

    if last_err() then
        clear_err()
        return false
    else
        return true
    end

execve
------

Execute an external program. Returns the exit code.

.. code-block:: lua

    execve("myprog", {"arg1", "arg2", "--arg", "3"})

hex
---

Hex encode a list of bytes.

.. code-block:: lua

    hex("\x6F\x68\x61\x69\x0A\x00")

hmac_md5
--------

Calculate an hmac with md5. Returns a binary array.

.. code-block:: lua

    hmac_md5("secret", "my authenticated message")

hmac_sha1
---------

Calculate an hmac with sha1. Returns a binary array.

.. code-block:: lua

    hmac_sha1("secret", "my authenticated message")

hmac_sha2_256
-------------

Calculate an hmac with sha2_256. Returns a binary array.

.. code-block:: lua

    hmac_sha2_256("secret", "my authenticated message")

hmac_sha2_512
-------------

Calculate an hmac with sha2_512. Returns a binary array.

.. code-block:: lua

    hmac_sha2_512("secret", "my authenticated message")

hmac_sha3_256
-------------

Calculate an hmac with sha3_256. Returns a binary array.

.. code-block:: lua

    hmac_sha3_256("secret", "my authenticated message")

hmac_sha3_512
-------------

Calculate an hmac with sha3_512. Returns a binary array.

.. code-block:: lua

    hmac_sha3_512("secret", "my authenticated message")

html_select
-----------

Parses an html document and returns the first element that matches the css
selector. The return value is a table with ``text`` being the inner text and
``attrs`` being a table of the elements attributes.

.. code-block:: lua

    csrf = html_select(html, 'input[name="csrf"]')
    token = csrf["attrs"]["value"]

html_select_list
----------------

Same as html_select_ but returns all matches instead of the
first one.

.. code-block:: lua

    html_select_list(html, 'input[name="csrf"]')

http_basic_auth
---------------

Sends a ``GET`` request with basic auth. Returns ``true`` if no ``WWW-Authenticate``
header is set and the status code is not ``401``.

.. code-block:: lua

    http_basic_auth("https://httpbin.org/basic-auth/foo/buzz", user, password)

http_mksession
--------------

Create a session object. This is similar to ``requests.Session`` in
python-requests and keeps track of cookies.

.. code-block:: lua

    session = http_mksession()

http_request
------------

Prepares an http request. The first argument is the session reference and
cookies from that session are copied into the request. After the request has
been sent, the cookies from the response are copied back into the session.

The next arguments are the ``method``, the ``url`` and additional options. Please
note that you still need to specify an empty table ``{}`` even if no options are
set. The following options are available:

- ``query`` - a map of query parameters that should be set on the url
- ``headers`` - a map of headers that should be set
- ``basic_auth`` - configure the basic auth header with ``{"user, "password"}``
- ``user_agent`` - overwrite the default user agent with a string
- ``json`` - the request body that should be json encoded
- ``form`` - the request body that should be form encoded
- ``body`` - the raw request body as string

.. code-block:: lua

    req = http_request(session, 'POST', 'https://httpbin.org/post', {
        json={
            user=user,
            password=password,
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp["status"] ~= 200 then return "invalid status code" end

http_send
---------

Send the request that has been built with http_request_. Returns a table with
the following keys:

- ``status`` - the http status code
- ``headers`` - a table of headers
- ``text`` - the response body as string

.. code-block:: lua

    req = http_request(session, 'POST', 'https://httpbin.org/post', {
        json={
            user=user,
            password=password,
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp["status"] ~= 200 then return "invalid status code" end

json_decode
-----------

Decode a lua value from a json string.

.. code-block:: lua

    json_decode("{\"data\":{\"password\":\"fizz\",\"user\":\"bar\"},\"list\":[1,3,3,7]}")

json_encode
-----------

Encode a lua value to a json string. Note that empty tables are encoded to an
empty object ``{}`` instead of an empty list ``[]``.

.. code-block:: lua

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

last_err
--------

Returns ``nil`` if no error has been recorded, returns a string otherwise.

.. code-block:: lua

    if last_err() then return end

ldap_bind
---------

Connect to an ldap server and try to authenticate with the given user.

.. code-block:: lua

    ldap_bind("ldaps://ldap.example.com/",
        "cn=\"" .. ldap_escape(user) .. "\",ou=users,dc=example,dc=com", password)

ldap_escape
-----------

Escape an attribute value in a relative distinguished name.

.. code-block:: lua

    ldap_escape(user)

ldap_search_bind
----------------

Connect to an ldap server, log into a search user, search for the target user
and then try to authenticate with the first DN that was returned by the search.

.. code-block:: lua

    ldap_search_bind("ldaps://ldap.example.com/",
        -- the user we use to find the correct DN
        "cn=search_user,ou=users,dc=example,dc=com", "searchpw",
        -- base DN we search in
        "dc=example,dc=com",
        -- the user we test
        user, password)

md5
---

Hash a byte array with md5 and return the results as bytes.

.. code-block:: lua

    hex(md5("\x00\xff"))

mysql_connect
-------------

Connect to a mysql database and try to authenticate with the provided
credentials. Returns a mysql connection on success.

.. code-block:: lua

    sock = mysql_connect("127.0.0.1", 3306, user, password)

mysql_query
-----------

Run a query on a mysql connection. The 3rd parameter is for prepared
statements.

.. code-block:: lua

    rows = mysql_query(sock, 'SELECT VERSION(), :foo as foo', {
        foo='magic'
    })

print
-----

Prints the value of a variable. Please note that this bypasses the regular
writer and may interfer with the progress bar. Only use this for debugging.

.. code-block:: lua

    print({
        data={
            user=user,
            password=password
        }
    })

rand
----

Returns a random ``u32`` with a minimum and maximum constraint. The return
value can be greater or equal to the minimum boundary, and always lower than
the maximum boundary. This function has not been reviewed for cryptographic
security.

.. code-block:: lua

    rand(0, 256)

randombytes
-----------

Generate the specified number of random bytes.

.. code-block:: lua

    randombytes(16)

sha1
----

Hash a byte array with sha1 and return the results as bytes.

.. code-block:: lua

    hex(sha1("\x00\xff"))

sha2_256
--------

Hash a byte array with sha2_256 and return the results as bytes.

.. code-block:: lua

    hex(sha2_256("\x00\xff"))

sha2_512
--------

Hash a byte array with sha2_512 and return the results as bytes.

.. code-block:: lua

    hex(sha2_512("\x00\xff"))

sha3_256
--------

Hash a byte array with sha3_256 and return the results as bytes.

.. code-block:: lua

    hex(sha3_256("\x00\xff"))

sha3_512
--------

Hash a byte array with sha3_512 and return the results as bytes.

.. code-block:: lua

    hex(sha3_512("\x00\xff"))

sleep
-----

Pauses the thread for the specified number of seconds. This is mostly used to
debug concurrency.

.. code-block:: lua

    sleep(3)

sock_connect
------------

Create a tcp connection.

.. code-block:: lua

    sock = sock_connect("127.0.0.1", 1337)

sock_send
---------

Send data to the socket.

.. code-block:: lua

    sock_send(sock, "hello world")

sock_recv
---------

Receive up to 4096 bytes from the socket.

.. code-block:: lua

    x = sock_recv(sock)

sock_sendline
-------------

Send a string to the socket. A newline is automatically appended to the string.

.. code-block:: lua

    sock_sendline(sock, line)

sock_recvline
-------------

Receive a line from the socket. The line includes the newline.

.. code-block:: lua

    x = sock_recvline(sock)

sock_recvall
------------

Receive all data from the socket until EOF.

.. code-block:: lua

    x = sock_recvall(sock)

sock_recvline_contains
----------------------

Receive lines from the server until a line contains the needle, then return
this line.

.. code-block:: lua

    x = sock_recvline_contains(sock, needle)

sock_recvline_regex
-------------------

Receive lines from the server until a line matches the regex, then return this
line.

.. code-block:: lua

    x = sock_recvline_regex(sock, "^250 ")

sock_recvn
----------

Receive exactly n bytes from the socket.

.. code-block:: lua

    x = sock_recvn(sock, 4)

sock_recvuntil
--------------

Receive until the needle is found, then return all data including the needle.

.. code-block:: lua

    x = sock_recvuntil(sock, needle)

sock_sendafter
--------------

Receive until the needle is found, then write data to the socket.

.. code-block:: lua

    sock_sendafter(sock, needle, data)

sock_newline
------------

Overwrite the default `\n` newline.

.. code-block:: lua

    sock_newline(sock, "\r\n")

Wrapping python scripts
-----------------------

The badtouch runtime is still very bare bones, so you might have to shell
out to your regular python script occasionally. Your wrapper may look like this:

.. code-block:: lua

    descr = "example.com"

    function verify(user, password)
        ret = execve("./docs/test.py", {user, password})
        if last_err() then return end

        if ret == 2 then
            return "script signaled an exception"
        end

        return ret == 0
    end

Your python script may look like this:

.. code-block:: python

    import sys

    try:
        if sys.argv[1] == "foo" and sys.argv[2] == "bar":
            # correct credentials
            sys.exit(0)
        else:
            # incorrect credentials
            sys.exit(1)
    except:
        # signal an exception
        # this requeues the attempt instead of discarding it
        sys.exit(2)
