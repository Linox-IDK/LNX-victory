//! `VICTORY_NATIVE_BVH=1` (LNX, off by default, with `VICTORY_BVH_HOOK=1`):
//! beside the console builder, the same ray-tracing structures are built
//! with the host's own DXR, from the same inputs (the title's standard
//! D3D12 descriptions). Nothing reads them yet: this is the step that makes
//! them and checks they build (sizes, counts); the translated shaders keep
//! reading the console's structures.
//!
//! Each structure is kept by its title address (the description's
//! destination): bottom levels from the title's triangles (its index,
//! vertex and transform buffers named through their host copies), top
//! levels from its instances, read from title memory with each instance's
//! bottom level replaced by the host one built for that title address.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_SAMPLE_DESC};
use windows::core::Interface;

use crate::object::Object;

/// Whether host structures are built (`VICTORY_NATIVE_BVH=1`).
pub fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_NATIVE_BVH").is_some_and(|v| v == "1"))
}

/// One triangle geometry, as the title describes it (title addresses).
#[derive(Debug, Clone, Copy)]
pub struct Triangles {
    pub opaque: bool,
    pub transform: u64,
    pub index_format: u32,
    pub vertex_format: u32,
    pub index_count: u32,
    pub vertex_count: u32,
    pub index_buffer: u64,
    pub vertex_buffer: u64,
    pub stride: u64,
}

/// One build the title asked for (title addresses).
#[derive(Debug, Clone)]
pub enum Build {
    Bottom {
        dest: u64,
        flags: u32,
        geometries: Vec<Triangles>,
    },
    Top {
        dest: u64,
        flags: u32,
        count: u32,
        instances: u64,
    },
}

/// A host structure and its size.
struct Structure {
    buffer: ID3D12Resource,
    size: u64,
}

/// The host structures by title address, and the shared scratch buffer.
struct Store {
    by_title: HashMap<u64, Structure>,
    scratch: Option<Structure>,
}

static STORE: Mutex<Option<Store>> = Mutex::new(None);

/// Counts since the last summary: built bottom/top levels, skipped ones
/// (an input without a host copy, an unknown bottom level), failures.
static BUILT_BOTTOM: AtomicU64 = AtomicU64::new(0);
static BUILT_TOP: AtomicU64 = AtomicU64::new(0);
static SKIPPED: AtomicU64 = AtomicU64::new(0);
static MISSING_BLAS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

/// A host GPU address for the title address `address` with `len` bytes
/// behind it, its host copy brought up to date before the list runs.
fn host_address(list: &crate::host::HostList, address: u64, len: u64) -> Option<u64> {
    if address == 0 {
        return Some(0);
    }
    let shadow = crate::shadow::find_any(address, len.max(1))?;
    shadow.refresh(list, address, len.max(1));
    Some(shadow.gpu(address))
}

fn buffer(
    device: &ID3D12Device,
    size: u64,
    state: D3D12_RESOURCE_STATES,
) -> Option<ID3D12Resource> {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Width: size.max(256),
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
        ..Default::default()
    };
    let mut out: Option<ID3D12Resource> = None;
    // SAFETY: plain creation of a buffer.
    let made = unsafe {
        device.CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &desc, state, None, &mut out)
    };
    if let Err(e) = made {
        tracing::warn!(error = %e, size, "native bvh: no buffer");
        return None;
    }
    out
}

/// The title's flags without an update (we build each time from scratch).
fn flags(title: u32) -> D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAGS {
    D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAGS((title & 0x1f) as i32)
}

/// Builds `builds` into the host list behind the title list `list`.
pub fn build(list: *const Object, builds: &[Build]) {
    if !on() || builds.is_empty() {
        return;
    }
    let Some(device) = crate::external::host_device() else {
        return;
    };
    let Ok(device5) = device.cast::<ID3D12Device5>() else {
        static SAID: std::sync::Once = std::sync::Once::new();
        SAID.call_once(|| tracing::warn!("native bvh: the host device has no DXR (ID3D12Device5)"));
        return;
    };
    crate::list::with_state(list, |host, state| {
        let Ok(l4) = host.list.cast::<ID3D12GraphicsCommandList4>() else {
            return;
        };
        let mut store = STORE.lock().unwrap_or_else(|e| e.into_inner());
        let store = store.get_or_insert_with(|| Store {
            by_title: HashMap::new(),
            scratch: None,
        });
        for b in builds {
            build_one(&device, &device5, &l4, host, store, b);
        }
        state.replay(host);
    });
    summary();
}

fn build_one(
    device: &ID3D12Device,
    device5: &ID3D12Device5,
    l4: &ID3D12GraphicsCommandList4,
    host: &crate::host::HostList,
    store: &mut Store,
    b: &Build,
) {
    // The inputs, with host addresses.
    let mut geometry = Vec::new();
    let mut instance_upload: Option<ID3D12Resource> = None;
    let (dest, inputs) = match b {
        Build::Bottom {
            dest,
            flags: f,
            geometries,
        } => {
            for g in geometries {
                let index_size: u64 = match g.index_format {
                    57 => 2, // R16_UINT
                    42 => 4, // R32_UINT
                    _ => 0,
                };
                let ib_len = u64::from(g.index_count) * index_size;
                let vb_len = u64::from(g.vertex_count) * g.stride;
                let (Some(ib), Some(vb), Some(tr)) = (
                    if g.index_buffer == 0 {
                        Some(0)
                    } else {
                        host_address(host, g.index_buffer, ib_len)
                    },
                    host_address(host, g.vertex_buffer, vb_len),
                    host_address(host, g.transform, 48),
                ) else {
                    SKIPPED.fetch_add(1, Ordering::Relaxed);
                    return;
                };
                geometry.push(D3D12_RAYTRACING_GEOMETRY_DESC {
                    Type: D3D12_RAYTRACING_GEOMETRY_TYPE_TRIANGLES,
                    Flags: if g.opaque {
                        D3D12_RAYTRACING_GEOMETRY_FLAG_OPAQUE
                    } else {
                        D3D12_RAYTRACING_GEOMETRY_FLAG_NONE
                    },
                    Anonymous: D3D12_RAYTRACING_GEOMETRY_DESC_0 {
                        Triangles: D3D12_RAYTRACING_GEOMETRY_TRIANGLES_DESC {
                            Transform3x4: tr,
                            IndexFormat: DXGI_FORMAT(if ib == 0 {
                                0
                            } else {
                                g.index_format as i32
                            }),
                            VertexFormat: DXGI_FORMAT(g.vertex_format as i32),
                            IndexCount: if ib == 0 { 0 } else { g.index_count },
                            VertexCount: g.vertex_count,
                            IndexBuffer: ib,
                            VertexBuffer: D3D12_GPU_VIRTUAL_ADDRESS_AND_STRIDE {
                                StartAddress: vb,
                                StrideInBytes: g.stride,
                            },
                        },
                    },
                });
            }
            (
                *dest,
                D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS {
                    Type: D3D12_RAYTRACING_ACCELERATION_STRUCTURE_TYPE_BOTTOM_LEVEL,
                    Flags: flags(*f),
                    NumDescs: geometry.len() as u32,
                    DescsLayout: D3D12_ELEMENTS_LAYOUT_ARRAY,
                    Anonymous: D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS_0 {
                        pGeometryDescs: geometry.as_ptr(),
                    },
                },
            )
        }
        Build::Top {
            dest,
            flags: f,
            count,
            instances,
        } => {
            // The instances from title memory, each bottom level replaced by ours.
            let n = *count as usize;
            let mut raw = vec![0u8; n * 64];
            if n > 0 && !crate::host::read_into(*instances as *const u8, &mut raw) {
                SKIPPED.fetch_add(1, Ordering::Relaxed);
                return;
            }
            for i in 0..n {
                let at = i * 64 + 56;
                let title = u64::from_le_bytes(raw[at..at + 8].try_into().unwrap_or_default());
                let host_va = store.by_title.get(&title).map_or(0, |s| {
                    // SAFETY: a live buffer.
                    unsafe { s.buffer.GetGPUVirtualAddress() }
                });
                if host_va == 0 {
                    MISSING_BLAS.fetch_add(1, Ordering::Relaxed);
                    // An instance with no structure: masked out.
                    raw[i * 64 + 48..i * 64 + 52].fill(0);
                }
                raw[at..at + 8].copy_from_slice(&host_va.to_le_bytes());
            }
            let upload = crate::host::host().and_then(|h| h.upload_buffer((n * 64).max(64) as u64));
            let Some((upload, data)) = upload else {
                SKIPPED.fetch_add(1, Ordering::Relaxed);
                return;
            };
            // SAFETY: the mapped upload buffer holds at least `raw`.
            unsafe { std::ptr::copy_nonoverlapping(raw.as_ptr(), data, raw.len()) };
            // SAFETY: a live buffer.
            let va = unsafe { upload.GetGPUVirtualAddress() };
            instance_upload = Some(upload);
            (
                *dest,
                D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS {
                    Type: D3D12_RAYTRACING_ACCELERATION_STRUCTURE_TYPE_TOP_LEVEL,
                    Flags: flags(*f),
                    NumDescs: *count,
                    DescsLayout: D3D12_ELEMENTS_LAYOUT_ARRAY,
                    Anonymous: D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS_0 {
                        InstanceDescs: va,
                    },
                },
            )
        }
    };
    let mut info = D3D12_RAYTRACING_ACCELERATION_STRUCTURE_PREBUILD_INFO::default();
    // SAFETY: inputs naming live host memory (or none).
    unsafe { device5.GetRaytracingAccelerationStructurePrebuildInfo(&inputs, &mut info) };
    if info.ResultDataMaxSizeInBytes == 0 {
        SKIPPED.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let result_size = info.ResultDataMaxSizeInBytes.next_multiple_of(256);
    let scratch_size = info.ScratchDataSizeInBytes.next_multiple_of(256);
    // The result buffer: kept for this title address, made bigger when needed.
    let fits = store
        .by_title
        .get(&dest)
        .is_some_and(|s| s.size >= result_size);
    if !fits {
        let Some(buf) = buffer(
            device,
            result_size,
            D3D12_RESOURCE_STATE_RAYTRACING_ACCELERATION_STRUCTURE,
        ) else {
            return;
        };
        BYTES.fetch_add(result_size, Ordering::Relaxed);
        if let Some(old) = store.by_title.insert(
            dest,
            Structure {
                buffer: buf,
                size: result_size,
            },
        ) {
            // The list may still read the old one: it holds it until done.
            host.keep(&old.buffer);
        }
    }
    let scratch_fits = store
        .scratch
        .as_ref()
        .is_some_and(|s| s.size >= scratch_size);
    if !scratch_fits {
        let Some(buf) = buffer(device, scratch_size, D3D12_RESOURCE_STATE_UNORDERED_ACCESS) else {
            return;
        };
        if let Some(old) = store.scratch.replace(Structure {
            buffer: buf,
            size: scratch_size,
        }) {
            host.keep(&old.buffer);
        }
    }
    let (Some(result), Some(scratch)) = (store.by_title.get(&dest), store.scratch.as_ref()) else {
        return;
    };
    // SAFETY: live buffers.
    let desc = unsafe {
        D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_DESC {
            DestAccelerationStructureData: result.buffer.GetGPUVirtualAddress(),
            Inputs: inputs,
            SourceAccelerationStructureData: 0,
            ScratchAccelerationStructureData: scratch.buffer.GetGPUVirtualAddress(),
        }
    };
    // SAFETY: an open list; the description names live buffers.
    unsafe { l4.BuildRaytracingAccelerationStructure(&desc, None) };
    // The shared scratch and this result are ready before the next build reads them.
    let barriers = [uav_barrier(&scratch.buffer), uav_barrier(&result.buffer)];
    // SAFETY: an open list.
    unsafe { l4.ResourceBarrier(&barriers) };
    host.keep(&result.buffer);
    host.keep(&scratch.buffer);
    if let Some(upload) = instance_upload {
        host.keep(&upload);
    }
    match b {
        Build::Bottom { .. } => BUILT_BOTTOM.fetch_add(1, Ordering::Relaxed),
        Build::Top { .. } => BUILT_TOP.fetch_add(1, Ordering::Relaxed),
    };
}

fn uav_barrier(r: &ID3D12Resource) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            // SAFETY: a borrowed copy of the interface pointer, not released by the barrier.
            UAV: std::mem::ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                pResource: unsafe { std::mem::transmute_copy(r) },
            }),
        },
    }
}

/// Every 5 s: what was built since.
fn summary() {
    static LAST: Mutex<Option<std::time::Instant>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    let now = std::time::Instant::now();
    if last.is_some_and(|t| now.duration_since(t).as_secs() < 5) {
        return;
    }
    *last = Some(now);
    let kept = STORE
        .lock()
        .ok()
        .and_then(|s| s.as_ref().map(|s| s.by_title.len()))
        .unwrap_or(0);
    tracing::info!(
        bottom = BUILT_BOTTOM.swap(0, Ordering::Relaxed),
        top = BUILT_TOP.swap(0, Ordering::Relaxed),
        skipped = SKIPPED.swap(0, Ordering::Relaxed),
        instances_without_ours = MISSING_BLAS.swap(0, Ordering::Relaxed),
        structures_kept = kept,
        mb_made = BYTES.load(Ordering::Relaxed) >> 20,
        "native bvh: host structures built in the last 5 s"
    );
}
