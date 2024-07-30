## How to contribute

1. Fork the repo
2. Make sure there is an open issue or a task in the [project](https://github.com/users/radumarias/projects/5)
3. **Assign it to you and move it in the corresponding status column as you are working on it**
4. Make the changes in your fork
5. If you add new `.rs` files in `examples` member add all these [lines](https://github.com/radumarias/rfs/blob/main/src/lib.rs#L1-L17) as first ones
6. If you add new packages to the workspace add all these [lines](https://github.com/radumarias/rfs/blob/main/src/lib.rs#L1-L17) to it's `lib.rs`
   and to any `bin` files (
   like `main.rs` or other files declared as `[[bin]]`)
7. Add tests for your changes, if applicable
8. `cargo fmt --all`, you can configure your **IDE** to do this on
   save, [RustRover](https://www.jetbrains.com/help/rust/rustfmt.html)
   and [VSCode](https://code.visualstudio.com/docs/languages/rust#_formatting)
19. Create a **PR** back to **main** repo to the `main` branch
10. Monitor the checks (GitHub actions run)
11. Respond to any comments
12. In the end, ideally, it will be merged to `main`
