# Components

As mentioned [at the start of the chapter](README.md#component), Async UI
does not have any specific interface that "components" have to conform to.
A "component" is just some piece of code that can be used to render UI.

Here are two common forms of "component"

## Async Functions

This code from the [previous subchapter](./4-siblings.md) is somewhat complicated.
```rust
{{ #include ../../../examples/guide-project/src/building_ui/components.rs:many-spans }}
```

We should split it into smaller parts that are easier to understand.

To do this, we'll isolate two of the Futures into a separate async functions
(remember that Rust async functions return Future objects).

```rust
{{ #include ../../../examples/guide-project/src/building_ui/components.rs:many-spans-components }}
```

Our overall UI function is now just
```rust
{{ #include ../../../examples/guide-project/src/building_ui/components.rs:many-spans-componentified }}
```

Observe that the components we made are just plain Rust async function.
Component functions can do all the things regular async functions can do:
take arguments, borrow things, be generic, etc.

## Types with Async `.render()` Method

Components can also come in the form of a type.
```rust
{{ #include ../../../examples/guide-project/src/building_ui/components.rs:type-component }}
```
Calling the `.render()` method returns a Future. Running that Future puts the UI
on the screen.

You've seen this before! all the HTML components we've worked with -
`Div`, `Input`, `Button`, etc. - are types with `.render(_)` method.

How is this more useful than plain asnyc functions?
It allows us to *modify* the element, as we'll see in the next chapter...