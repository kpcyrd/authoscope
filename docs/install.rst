Installation
============

If available, please prefer the package shipped by your linux distribution.

Archlinux
---------

.. code-block:: bash

    $ pacman -S badtouch

Mac OSX
-------

.. code-block:: bash

    $ brew install badtouch

Docker
------

.. code-block:: bash

    $ docker run --rm kpcyrd/badtouch --help

Source
------

To build from source, make sure you have rust_ and ``libssl-dev`` installed.

.. _rust: https://rustup.rs/

.. code-block:: bash

    $ git clone https://github.com/kpcyrd/badtouch
    $ cd badtouch
    $ cargo build
