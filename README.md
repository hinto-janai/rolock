# RoLock
[![Windows](https://github.com/hinto-janai/rolock/actions/workflows/windows.yml/badge.svg)](https://github.com/hinto-janai/rolock/actions/workflows/windows.yml) [![macOS](https://github.com/hinto-janai/rolock/actions/workflows/macos.yml/badge.svg)](https://github.com/hinto-janai/rolock/actions/workflows/macos.yml) [![Linux](https://github.com/hinto-janai/rolock/actions/workflows/linux.yml/badge.svg)](https://github.com/hinto-janai/rolock/actions/workflows/linux.yml) [![crates.io](https://img.shields.io/crates/v/rolock.svg)](https://crates.io/crates/rolock) [![docs.rs](https://docs.rs/rolock/badge.svg)](https://docs.rs/rolock)

Read Only Lock.

This is a wrapper around `Arc<RwLock<T>>` that only implements `RwLock::read()` operations.

## Usage
Create a normal `Arc<RwLock<T>>` in `thread_1`, send a `RoLock` to `thread_2`:
```rust
let rw = Arc::new(RwLock::new(0));     // Regular Arc<RwLock<T>>.
let ro = RoLock::new(&rw);             // Read Only Lock.

assert!(*rw.read().unwrap() == 0);     // This can read...
*rw.write().unwrap() = 1;              // and write.

std::thread::spawn(move|| {
	assert!(*ro.read().unwrap() == 1); // This one can only read.
});
```
- `thread_1` still has full read/write control
- `thread_2` can only `RoLock::read()`

This type guarantees at compile time that you cannot write because the function doesn't even exist:
```rust
let rw = Arc::new(RwLock::new(0));
let ro = RoLock::new(&rw);

ro.write(); // Compile error!
```

Since the inner field of `RoLock` (`self.0`) is private, you can't call `RwLock::write` directly either:
```rust
let rw = Arc::new(RwLock::new(0));
let ro = RoLock::new(&rw);

ro.0.write(); // Compile error!
```
