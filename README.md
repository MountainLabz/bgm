# binary-greedy-meshing
This is a modification of "Originally a port of [Binary Greedy Meshing v2](https://github.com/cgerikj/binary-greedy-meshing) to Rust, with additional improvements such as support for transparent blocks." by Inspirateur.

## How to use
It works like the original, except it outputs quads as two u32s specifically for vertex pulling.

### Minimal example
```rust
use binary_greedy_meshing as bgm;
use std::collections::BTreeSet;

fn main() {
    // This is a flattened 3D array of u16 in ZXY order, of size 64^3 
    // (it represents a 62^3-sized chunk that is padded with neighbor information)
    let mut voxels = [0; bgm::CS_P3];
    // Add 2 voxels at position 0;0;0 and 0;1;0
    voxels[bgm::pad_linearize(0, 0, 0)] = 1;
    voxels[bgm::pad_linearize(0, 1, 0)] = 1;
    // Contain useful buffers that can be cached and cleared 
    // with mesh_data.clear() to avoid re-allocation
    let mut mesh_data = bgm::MeshData::new();
    // Does the meshing, mesh_data.quads is the output
    // transparent block values are signaled by putting them in the BTreeSet
    bgm::mesh(&voxels, &mut mesh_data, BTreeSet::default());
}
```

### What to do with `mesh_data.quads`
`mesh_data.quads` is a vector of u64s (broken into u32s) each u64 encoding all the information of a quad in the following manner:
```rust
let data1 = (x as u32) | ((y as u32) << 6) | ((z as u32) << 12) | ((w as u32) << 18) | ((h as u32) << 24);
let data2 = (v_type as u32) | ((normal as u32) << 16);
```

The normal is used the gpu for vertex pulling instead of storing each of the quads in a different vector.

use vertex pulling.
vertex pulling is more efficent than instancing because you don't send 6 verts (which are expensive) you just send 1 u64.
the reason this is better is because instancing performs very badly on some gpu's. So its generally better.

## Performance
Benching the crate on Intel(R) Xeon(R) CPU E5-1650 v3 @ 3.50GHz:
- meshing (with transparency support): **400μs**

This is coherent with the 50-200μs range (without transparency) reported from the original C version of the library, as transparency incurrs a significant cost in the hidden face culling phase.

The meshing is also ~10x faster than [block-mesh-rs](https://github.com/bonsairobo/block-mesh-rs) which took **~4.5ms** to greedy mesh a chunk on my machine.

*chunk sizes are 62^3 (64^3 with padding), this crate doesn't support other sizes.*
