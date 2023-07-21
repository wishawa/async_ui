# Your First Component

### Before You Start

Make sure you have the project set up as per [Project Setup](../preparations/3-project-setup.md).

## Putting Something on the Screen

Let's put an `<input>` on the screen!

```rust,noplayground
	// src/lib.rs

{{ #include ../../../examples/guide-project/src/building_ui/first_component.rs:input-in-app }}
```
Now run the application again
(`wasm-pack build --dev --target web && microserver`, as described in the
[Project Setup](../preparations/3-project-setup.md)),
you should see an empty input field.
![A webpage with just an empty input field](./1-input.png)

## Extracting it into a Component
Let's extract that single input field into a *component*.
We'll make the simplest form of component possible: an async function.

```rust,noplayground
	// src/lib.rs

{{ #include ../../../examples/guide-project/src/building_ui/first_component.rs:input-component }}
```