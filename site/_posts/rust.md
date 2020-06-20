+++
title = "Kernel Heap"
date  = "2016-04-11"
summary = "Doing some OS stuff in Rust"
tags = ["rust", "programming"]
+++

In the previous posts we created a frame allocator and a page table module. Now we are ready to create a kernel heap and a memory allocator. Thus, we will unlock `Box`, `Vec`, `BTreeMap`, and the rest of the `alloc` crate.

```rust
pub unsafe trait Alloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
    […] // about 13 methods with default implementations
}
```

```
pub unsafe trait Alloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
    […] // about 13 methods with default implementations
}
```

The `alloc` method should allocate a memory block with the size and alignment given through `Layout` parameter. The `deallocate` method should free such memory blocks again. Both methods are `unsafe`, as is the trait itself. This has different reasons:
