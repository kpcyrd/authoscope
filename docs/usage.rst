Usage
========

Options
-------

.. code-block:: text

    -n, --workers <workers>     The number of concurrent workers to run.
    -o, --output <output>       Write results to this file.
    -v, --verbose               Enable verbose output.
    -h, --help                  Prints help information.
    -V, --version               Prints version information.

Dictionary attack
-----------------

Try each password for each user with every script.

.. code-block:: bash

    badtouch dict <users> <passwords> [scripts]...

Credential confirmation
-----------------------

Load a list of credentials with the format ``user:password`` and verify them
with every script.

.. code-block:: bash

    badtouch creds <credentials> [scripts]...

Username enumeration
--------------------

Takes a list of username and verifies they exist on the system. This is still
executing the ``verify`` function with two arguments, but the password is set
to ``nil``. You may write a script that can do both by checking the password
for nil to detect in which mode the script is executed.

.. code-block:: bash

    badtouch enum <users> [scripts]...

Oneshot
-------

Test a single username-password combination using a specific script. This
command is also useful when developing a new script. If the password argument
is omitted, the script is executed in enumerate mode. If you want to use this
command in scripts, set ``-x`` so the exitcode is set to 2 if the credentials
are invalid.

.. code-block:: bash

    badtouch oneshot [-x] <script> <user> [password]
