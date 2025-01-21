## SpecStory

## Fixing Deprecated QdrantClientConfig in Rust (1/19/2025, 4:55:09 PM)

_**User**_

For the code present, we get this error:
```
use of deprecated struct `qdrant_client::client::QdrantClientConfig`: use new config at `qdrant_client::config::QdrantConfig` instead
```
How can I resolve this? If you propose a fix, please make it concise.

---

_**Assistant**_

I'll help you update the deprecated config usage. Here's the fix:

```rust:src/vector_store/mod.rs
// ... existing code ...
    pub async fn new() -> Result<Self> {
        let client = QdrantClient::from_url("http://localhost:6334")?;
        
        // Create collection if it doesn't exist
// ... existing code ...
```

The change replaces the deprecated `QdrantClientConfig` with the newer, simpler `QdrantClient::from_url` method. This achieves the same result but uses the updated API.

---



---

