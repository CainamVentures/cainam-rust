## SpecStory

## Resolving Undeclared Crate Error in Rust (1/21/2025, 9:40:38 AM)

_**User**_

For the code present, we get this error:
```
failed to resolve: use of undeclared crate or module `rig_core`
use of undeclared crate or module `rig_core`
```
How can I resolve this? If you propose a fix, please make it concise.

---

_**Assistant**_

Let me check the dependencies to help resolve this error.

---

_**Assistant**_



---

_**Assistant**_



I see the issue. While `rig-core` is correctly declared as a dependency in `Cargo.toml`, it's looking for it in a relative path `../rig-core`. Let me check if this directory exists.

---

_**Assistant**_



---

_**Assistant**_



I see the problem and solution. You have two options:

1. **Recommended Fix**: Update the workspace structure. Add this to your root `Cargo.toml`:
```toml
[workspace]
members = ["rig-core", "rig-birdeye"]
```

2. Alternative Fix: Update the dependency path in `rig-birdeye/Cargo.toml` to use the absolute path:
```toml
rig-core = { path = "../../rig-core" }
```

I recommend option 1 as it's the proper way to structure a Rust workspace with multiple crates. Would you like me to implement one of these fixes?

---



---

