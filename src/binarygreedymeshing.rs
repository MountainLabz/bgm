#![no_std]



use crate::lib::face;

use alloc::{boxed::Box, collections::BTreeSet, vec::Vec};
pub use face::*;
pub const CS: usize = 62;
const CS_2: usize = CS * CS;
pub const CS_P: usize = CS + 2;
pub const CS_P2: usize = CS_P * CS_P;
pub const CS_P3: usize = CS_P * CS_P * CS_P;

#[derive(Debug)]
pub struct MeshData {
    /// Single buffer for packed quads (normal | v_type | other data)
    pub quads: Vec<PackedQuad>,
    /// Internal buffers
    face_masks: Box<[u64]>,
    forward_merged: Box<[u8]>,
    right_merged: Box<[u8]>,
}

use bytemuck::{Pod, Zeroable};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct PackedQuad {
    data1: u32, // Lower 32 bits
    data2: u32, // Upper 32 bits
}

impl PackedQuad {
    pub fn new(x: usize, y: usize, z: usize, w: usize, h: usize, v_type: u16, normal: u16) -> Self {
        let data1 = (x as u32) | ((y as u32) << 6) | ((z as u32) << 12) | ((w as u32) << 18) | ((h as u32) << 24);
        let data2 = (v_type as u32) | ((normal as u32) << 16);
        Self { data1, data2 }
    }
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            face_masks: vec![0; CS_2 * 6].into_boxed_slice(),
            forward_merged: vec![0; CS_2].into_boxed_slice(),
            right_merged: vec![0; CS].into_boxed_slice(),
            quads: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.face_masks.fill(0);
        self.forward_merged.fill(0);
        self.right_merged.fill(0);
        self.quads.clear();
    }
}

#[inline]
fn face_value(v1: u16, v2: u16, transparents: &BTreeSet<u16>) -> u64 {
    (v2 == 0 || (v1 != v2 && transparents.contains(&v2))) as u64
}

pub fn mesh(voxels: &[u16], mesh_data: &mut MeshData, transparents: BTreeSet<u16>) {
    for a in 1..(CS_P - 1) {
        let a_cs_p = a * CS_P;
        for b in 1..(CS_P - 1) {
            let ab = (a_cs_p + b) * CS_P;
            let ba_index = (b - 1) + (a - 1) * CS;
            let ab_index = (a - 1) + (b - 1) * CS;
            for c in 1..(CS_P - 1) {
                let abc = ab + c;
                let v1 = voxels[abc];
                if v1 == 0 { continue; }
                mesh_data.face_masks[ba_index + 0 * CS_2] |= face_value(v1, voxels[abc + CS_P2], &transparents) << (c - 1);
                mesh_data.face_masks[ba_index + 1 * CS_2] |= face_value(v1, voxels[abc - CS_P2], &transparents) << (c - 1);
                mesh_data.face_masks[ab_index + 2 * CS_2] |= face_value(v1, voxels[abc + CS_P], &transparents) << (c - 1);
                mesh_data.face_masks[ab_index + 3 * CS_2] |= face_value(v1, voxels[abc - CS_P], &transparents) << (c - 1);
                mesh_data.face_masks[ba_index + 4 * CS_2] |= face_value(v1, voxels[abc + 1], &transparents) << c;
                mesh_data.face_masks[ba_index + 5 * CS_2] |= face_value(v1, voxels[abc - 1], &transparents) << c;
            }
        }
    }

    for face in 0..6 {
        let normal = match face {
            0 => 0b0001, // +Z
            1 => 0b0010, // -Z
            2 => 0b0100, // +Y
            3 => 0b1000, // -Y
            4 => 0b0001, // +X
            5 => 0b0010, // -X
            _ => unreachable!(),
        } as u16;

        for layer in 0..CS {
            let bits_location = layer * CS + face * CS_2;
            for forward in 0..CS {
                let mut bits_here = mesh_data.face_masks[forward + bits_location];
                if bits_here == 0 { continue; }
                while bits_here != 0 {
                    let bit_pos = bits_here.trailing_zeros() as usize;
                    let v_type = voxels[get_axis_index(face / 2, forward + 1, bit_pos + 1, layer + 1)];
                    let quad = PackedQuad::new(forward, layer, bit_pos, 1, 1, v_type, normal);
                    mesh_data.quads.push(quad);
                    bits_here &= !(1 << bit_pos);
                }
            }
        }
    }
}

#[inline]
fn get_axis_index(axis: usize, a: usize, b: usize, c: usize) -> usize {
    match axis {
        0 => b + (a * CS_P) + (c * CS_P2),
        1 => b + (c * CS_P) + (a * CS_P2),
        _ => c + (a * CS_P) + (b * CS_P2),
    }
}

pub fn pad_linearize(x: usize, y: usize, z: usize) -> usize {
    z + 1 + (x + 1)*CS_P + (y + 1)*CS_P2
}