This program is made just for trying async-await code in the current ecosystem.
It features the following capabilities:

 * do https requests
 * do multiple requests at a time, one per page
 * use async closures

The code was done synchronously first, and then moved to async with a suprisingly small amount of
changes.

### Difficulties on the way...

Please note that at the time of writing, 2019-08-13, the ecosystem wasn't ready.
Search the code for `TODO` to learn about workarounds/issues still present.

* `async || {}` is not yet ready, and needs to be move. This comes with the additional limitation that references can't be passed as argument, everything it sees must be owned.
