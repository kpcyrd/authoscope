Installation
============

If available, please prefer the package shipped by your linux distribution.

Archlinux
---------

.. code-block:: bash

    $ pacman -S authoscope

Mac OSX
-------

.. code-block:: bash

    $ brew install authoscope

Docker
------

.. code-block:: bash

    $ docker run --rm kpcyrd/authoscope --help

Source
------

To build from source, make sure you have rust_ and ``libssl-dev`` installed.

.. _rust: https://rustup.rs/

.. code-block:: bash

    $ git clone https://github.com/kpcyrd/authoscope
    $ cd authoscope
    $ cargo build
