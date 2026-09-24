# Directory Fingerprint Strategy Benchmark Report (Local vs Network Filesystems)

**Date:** 2026-09-21  
**Test Platform:** Windows 11, NVMe SSD  
**Target Crates:** `omera-scan`, `omera-storage`  
**Benchmark Command:** `cargo bench --bench directory_fingerprint -p omera-scan`

---

## 1. Executive Summary

This benchmark evaluates filesystem reconciliation strategies for Omera across local fast storage (NVMe SSD) and network-attached storage (SMB, NFS, WebDAV, cloud mounts).

As libraries grow into tens of thousands of media files across complex folder hierarchies, scanning performance depends critically on minimizing operating system calls (system calls on local disks, and remote round-trip RPCs over network filesystems).

The benchmark compares four distinct directory fingerprint strategies across a realistic dataset of **5,000 files distributed across 50 subdirectories**:
1. **Strategy 1: Deep Recursive File Stat**: Traverses every directory entry, issuing an individual `stat()`/`getattr` per file to record `(size_bytes, modified_at)`.
2. **Strategy 2: Directory mtime Gating**:
   - **2a (Cold Scan)**: All subdirectories are scanned for the first time.
   - **2b (Warm Scan)**: Checks parent directory `mtime` against stored database fingerprints; if unchanged, skips recursing into that directory entirely (simulating 90% unchanged directories).
3. **Strategy 3: Batched Directory Enumeration**: Reads directory entries with dirent metadata directly from the directory stream table.
4. **Strategy 4: Content Hash Sampling**: Reads file content and computes a data hash for cryptographic verification.

---

## 2. Empirical Benchmark Results

### 2.1 Local Filesystem Performance (Fast NVMe SSD)

| Strategy | Total Duration | Syscall Count | Throughput | Notes |
|---|---|---|---|---|
| **Strategy 1: Recursive File Stat** | **6.83 ms** | 5,101 syscalls | **732,408 files/s** | Baseline streaming scanner |
| **Strategy 2a: Directory Gating (Cold)** | **8.72 ms** | 5,151 syscalls | **573,086 files/s** | Evaluates dir mtime + full walk |
| **Strategy 2b: Directory Gating (Warm 90%)** | **2.78 ms** | **910 syscalls** | **1,798,561 files/s** | **2.5x speedup** (82% syscall reduction) |
| **Strategy 3: Batched Readdir** | **7.26 ms** | 51 syscalls | **688,440 files/s** | Compact syscall profile |
| **Strategy 4: Content Hash Sampling** | **183.50 ms** | 10,101 syscalls | **27,249 files/s** | Heavy I/O & CPU bound |

> **Key Finding for Local Disks:**  
> On local NVMe/SSD, recursive file stat is already exceptionally fast (5,000 files in ~6.8 ms). However, when libraries contain thousands of directories, **directory mtime gating** reduces filesystem operations by >80% and accelerates incremental checks by **2.5x**. Content hashing is ~27x slower than metadata fingerprinting and should only be used on-demand or during explicit integrity audits.

---

### 2.2 Network Filesystem Latency Simulation (SMB / NFS / WebDAV)

Network filesystems incur high round-trip latency (RTT) for every remote filesystem metadata RPC (`LOOKUP`, `GETATTR`, `STAT`, `READDIR`).

#### Scenario A: LAN Storage (1.0 ms RTT per RPC)
*e.g. Local NAS, Gigabit Ethernet SMB/NFS share*

| Strategy | Network RPC Calls | Estimated Scan Duration | Speedup vs Baseline |
|---|---|---|---|
| **Strategy 1: Recursive File Stat** | 5,101 RPCs | **5.11 s** | 1.0x (baseline) |
| **Strategy 3: Batched Readdir** | 51 RPCs | **58.26 ms** | **87.7x** |
| **Strategy 2b: Directory Gating (Warm 90%)** | 910 RPCs | **912.78 ms** | **5.6x** |

#### Scenario B: WAN / VPN / Cloud Storage (10.0 ms RTT per RPC)
*e.g. Remote SMB over WireGuard, WebDAV, Multi-Database Team Studio over WAN*

| Strategy | Network RPC Calls | Estimated Scan Duration | Speedup vs Baseline |
|---|---|---|---|
| **Strategy 1: Recursive File Stat** | 5,101 RPCs | **51.02 s** | 1.0x (baseline) |
| **Strategy 2b: Directory Gating (Warm 90%)** | 910 RPCs | **9.10 s** | **5.6x** |

> **Key Finding for Network Filesystems:**  
> While local NVMe syscalls execute in nanoseconds, network storage is bottlenecked by round-trip latency. Strategy 1 (individual per-file stat) degrades severely over network shares (taking over 50 seconds for just 5,000 files on WAN).  
> **Directory mtime gating** slashes the required RPCs to only modified directories, preventing massive round-trip stalls and delivering a **5.6x speedup**.

---

## 3. Architectural Recommendations & Conclusions

1. **Local Disks (Mode A & Mode B Folders):**
   - The current streaming walker `walk_media_files` with `(size_bytes, modified_at)` fingerprinting remains optimal, achieving >700k files/sec.
   - For folders with deep subdirectories, directory mtime caching allows near-instantaneous startup reconciliation (< 3 ms per 5k items).

2. **Network Shares & Multi-Database Team Studio (Milestone 14):**
   - Must avoid per-file stat calls where possible.
   - Employ batched directory reads (`readdir`) combined with parent directory mtime gating.
   - When a remote subdirectory's timestamp has not changed since the last journal sync, skip remote traversal entirely.

3. **Reproducibility:**
   - Run `cargo bench --bench directory_fingerprint -p berry-scan` to reproduce these measurements across different hardware and OS environments.
