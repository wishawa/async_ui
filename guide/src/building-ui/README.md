# Building the UI

This chapter describes how to put things on the screen.
We'll cover how to make UI elements, and how to arrange and nest them.

### Some Terminology

We'll be talking about *components* and *elements*. Let's define them first.

#### Element
An *element* is an HTML node. For example:
* A `<button>` is an element.
* An [HTML Text Node](https://developer.mozilla.org/en-US/docs/Web/API/Text) is an element.
* A `<div>...</div>` is an element.

#### Component
There is no fixed interface for what a *component* looks like in Async UI.

The term "component" will be used to refer to any piece of code that can
be used to put some UI elements on the screen.

By the end of this chapter, you will see some common forms of components.