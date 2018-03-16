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
