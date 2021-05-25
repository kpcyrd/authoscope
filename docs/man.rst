authoscope
========

authoscope (formerly badtouch) is a scriptable network authentication cracker.
While the space for common service bruteforce is already very well saturated,
you may still end up writing your own python scripts when testing credentials
for web applications.

The scope of authoscope is specifically cracking custom services. This is done
by writing scripts that are loaded into a lua runtime. Those scripts represent
a single service and provide a ``verify(user, password)`` function that returns
either true or false. Concurrency, progress indication and reporting is
magically provided by the authoscope runtime.

.. toctree::
   :maxdepth: 3
   :glob:

   usage
   scripting
   config
