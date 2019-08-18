
#### Description

**I'm submitting a ...**  (check one with "x")

feature request / design doc

About a year ago, I contributed a simple router example to this project. Along side it, I also submitted a more complicated, higher-level router implementation that was too buggy to include in the project. It relied on some methods that should probably have been private at the time, but appear to be completely removed now. 

I would like to take a stab at making another React-Router-alike router for Yew.

I think some modifications to Yew might be needed to allow this to work, but first I'll start with an ideal end-product:
```
<Router>
    <Route
         path = {"/path/:id"}
         exact
         component = {Component1} // How would this be constrained to only allow Components with `Properties=RouterProps`?
    >
    <Route
         path = {"/otherPath/:number"}
         render = { |routerProps |  html!{
             <div> On page {routerProps.captures()["number"]} </div>
         }}
    />
   <Route component = Component2 /> // Routes if no others match
</Router>
```
This is obviously already blocked on https://github.com/DenisKolodin/yew/issues/537. 
But there may be other blockers I would like to explore in this document.


### Design


##### Router - Route component communication
Following implementation of 537, the `Router` Component would have its `Vec` of child components.
It would be required that the `Router` only communicate with its children, ideally without having to pass any props directly.

To avoid additional Props in the `Route` Components, an Agent could be used to communicate between the `Router` and the `Route`s.

In the case of multiple nested `Router`s interfering with each other, a bool present in every `Route` component could used to signal to avoid trying to route again after a route change.
This bool could be reset by the Agent broadcasting a reset message.

Either that solution, or a network of agents is able to communicate to assign id-groups to the Router and Routes, so they only communicate amongst themselves.


##### Specification of Render block.

This is simple enough, just take a callback that returns Html<Route>

##### Specification of Component

I don't know how to do this.
It could in theory be handled like the render section is, like the following: 
```
component = {html!{<Component1/>}}
```
Somehow, `RouteProps` would have to be injected into this component should it support it.

Either that, or 
```
<Route
...
>
    <Component1/>
</Route>
```
And the RouterProps are injected into the component again.


There isn't a very easy to use `Component::new()` function, because `VComp::new()` requires a ScopeHolder.
Ideally, if there was a way to easily use `VComp::new()` the following could be another option:
```
<Route<Component1> path=... />
```
Getting a a ScopeHolder to use with this would be a prerequisite though, and I don't know how to do that at the level of abstraction the router would be implemented at.

##### Requirement that Components take RouterProps as their Properties

Would there be a way to mandate that any component that appears as a child of or a field within a `Route` object must have `Properties=RouterProps` as a trait attribute?
How would this square with the existing syntax for the html!{} macro?

/** This line is dumb and misinformed, but it is the only current way to imagine how to accomplish this.*/
I would assume this would involve some form of coercing/casting the component=equals to this trait and hoping it doesn't fail. 



