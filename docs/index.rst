badtouch
========

badtouch is a scriptable network authentication cracker. While the space for
common service bruteforce is already very_ well_ saturated_, you may still end
up writing your own python scripts when testing credentials for web
applications.

.. _very: https://nmap.org/ncrack/
.. _well: https://github.com/vanhauser-thc/thc-hydra
.. _saturated: https://github.com/jmk-foofus/medusa

The scope of badtouch is specifically cracking custom services. This is done by
writing scripts that are loaded into a lua runtime. Those scripts represent a
single service and provide a ``verify(user, password)`` function that returns
either true or false. Concurrency, progress indication and reporting is
magically provided by the badtouch runtime.

.. image:: https://asciinema.org/a/Ke5rHVsz5sJePNUK1k0ASAvuZ.png
   :target: https://asciinema.org/a/Ke5rHVsz5sJePNUK1k0ASAvuZ

Getting Started
---------------

.. toctree::
   :maxdepth: 3
   :glob:

   install
   usage
   scripting
   config
