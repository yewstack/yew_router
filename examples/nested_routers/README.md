# Nesting Example

Currently this example exemplifies a bug in either the router implementation or in Yew.

In short, something in yew allows the component to run its `destroy` method, have it be removed from the view as it is written to the DOM, but not actually fully destroy it.
It persists.
When switching back and forth between the A and B components, the PageNotFound component will announce that it has been destroyed for every instance that has been created, even though it should only announce this once.


### Demonstration.
* Run this app using `cargo web start`
* Open the JS Console.
* Click the A and B buttons.
* Observe in the console a growing number of log messages detailing that the PageNotFound has been destroyed every time you switch from A to B.
