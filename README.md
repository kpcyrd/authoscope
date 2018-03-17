# badtouch [![Build Status][travis-img]][travis] [![Crates.io][crates-img]][crates]

[travis-img]:   https://travis-ci.org/kpcyrd/badtouch.svg?branch=master
[travis]:       https://travis-ci.org/kpcyrd/badtouch
[crates-img]:   https://img.shields.io/crates/v/badtouch.svg
[crates]:       https://crates.io/crates/badtouch

badtouch is a scriptable network authentication cracker. While the space for
common service bruteforce is already [very][ncrack] [well][hydra]
[saturated][medusa], you may still end up writing your own python scripts when
testing credentials for web applications.

[ncrack]: https://nmap.org/ncrack/
[hydra]: https://github.com/vanhauser-thc/thc-hydra
[medusa]: https://github.com/jmk-foofus/medusa

The scope of badtouch is specifically cracking custom services. This is done by
writing scripts that are loaded into a lua runtime. Those scripts represent a
single service and provide a `verify(user, password)` function that returns
either true or false. Concurrency, progress indication and reporting is
magically provided by the badtouch runtime.

## Reference
- [execve](#execve)
- [http_basic_auth](#http_basic_auth)
- [mysql_connect](#mysql_connect)
- [rand](#rand)
- [sleep](#sleep)
- [Examples](/scripts)
- [Wrapping python scripts](#wrapping-python-scripts)

### execve
Execute an external program. Returns the exit code.
```lua
execve("myprog", {"arg1", "arg2", "--arg", "3"})
```

### http_basic_auth
Sends a `GET` request with basic auth. Returns `true` if no `WWW-Authenticate`
header is set and the status code is not `401`.
```lua
http_basic_auth("https://httpbin.org/basic-auth/foo/buzz", user, password)
```

### mysql_connect
Connect to a mysql database and try to authenticate with the provided
credentials. Returns `true` on success.
```lua
mysql_connect("127.0.0.1", 3306, user, password)
```

### rand
Returns a random `u32` with a minimum and maximum constraint. The return value
can be greater or equal to the minimum boundary, and always lower than the
maximum boundary. This function has not been reviewed for cryptographic
security.
```lua
rand(0, 256)
```

### sleep
Pauses the thread for the specified number of seconds. This is mostly used to
debug concurrency.
```lua
sleep(3)
```

## Wrapping python scripts

The badtouch runtime is still extremely bare bones, so you might have to shell
out to your regular python script occasionally. Your wrapper my look like this:

```lua
descr = "example.com"

function verify(user, password)
    return execve("./docs/test.sh", {user, password}) == 0
end
```

Your python script may look like this:

```python
import sys

if sys.argv[1] == "foo" and sys.argv[2] == "bar":
    # correct credentials
    exit(0)
else:
    # incorrect credentials
    exit(1)
```

# License

GPLv3+
