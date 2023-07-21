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
A *component* is something that puts element(s) on the screen.

There is no fixed interface for what a component looks like in Async UI.
In this chapter we will cover different ways of making components.