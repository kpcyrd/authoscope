Configuration
=============

You can place a config file at ``~/.config/authoscope.toml`` to set some defaults.

Global user agent
-----------------

.. code-block:: toml

    [runtime]
    user_agent = "w3m/0.5.3+git20180125"

RLIMIT_NOFILE
-------------

.. code-block:: toml

    [runtime]
    # requires CAP_SYS_RESOURCE
    # sudo setcap 'CAP_SYS_RESOURCE=+ep' /usr/bin/authoscope
    rlimit_nofile = 64000
