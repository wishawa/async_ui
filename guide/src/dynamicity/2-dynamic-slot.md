# The DynamicSlot Component

The previous subchapter covered dynamically *updating* HTML elements.
Now we'll be *adding* and *removing* elements.

The [`DynamicSlot` component](https://docs.rs/async_ui_web/latest/async_ui_web/components/struct.DynamicSlot.html)
provided by Async UI acts like a "slot".
You can dynamically set what Future runs in the slot.

In this example, we will display a button for 3 seconds,
display some text for 3 seconds, and then display nothing.
```rust
{{ #include ../../../examples/guide-project/src/dynamicity/dynamic_slot.rs:example }}
```

> #### It's not magic
> If you're familiar with hand-implementing Futures, take a look at the
> [source code of DynamicSlot](https://github.com/wishawa/async_ui/blob/main/async_ui_web/src/components/dynamic_slot.rs).
> 
> You'll see that it's no private-API-powered magic;
> it's just general async Rust code. You can even implement it yourself!

## Extra
Can you implement the example above without `DynamicSlot`?
Hint: there is [`race`](https://docs.rs/async_ui_web/latest/async_ui_web/fn.race.html),
which is like `join`, but completes as soon as the first Future completes.

<details>
<summary>Click to view solution</summary>

Let's make a helper function first
```rust
{{ #include ../../../examples/guide-project/src/dynamicity/dynamic_slot.rs:extra-quiz-helper }}
```

And now our main UI
```rust
{{ #include ../../../examples/guide-project/src/dynamicity/dynamic_slot.rs:extra-quiz }}
```

</details>