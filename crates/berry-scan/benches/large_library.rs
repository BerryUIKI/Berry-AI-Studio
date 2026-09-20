//! Reproducible large-library performance benchmarks for 1k, 10k, and 50k items.
//!
//! Measures:
//! 1. Warm SQLite database time-to-first-usable gallery page (first 400 items + window total)
//! 2. Deep pagination query latency (offset paging vs keyset cursor) across 1k, 10k, 50k datasets
//! 3. 50k thumbnail manifest synchronization / legacy adoption time and throughput

use berry_domain::{
    Container, ExtractedMetadata, FileSortField, ImageFile, MetadataFormat, SearchCriteria,
    SortDirection,
};
use berry_scan::synchronize_thumbnail_manifest;
use berry_storage::Database;
use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;

fn generate_mock_files(folder_id: i64, start_index: usize, count: usize) -> Vec<ImageFile> {
    let samplers = ["Euler a", "DPM++ 2M Karras", "DDIM", "Euler"];
    let models = [
        "v1-5-pruned.safetensors",
        "sd_xl_base_1.0.safetensors",
        "anime_vae.safetensors",
    ];
    let prompts = [
        "masterpiece, best quality, 1girl, solo, cherry blossoms, sunset sky, cinematic lighting",
        "scenic landscape, majestic mountain peak, crystal blue lake, autumn foliage, photorealistic",
        "cyberpunk street, neon signs, rainy reflections, futuristic cityscape, ultra detailed",
        "fantasy castle on floating island, waterfalls, golden clouds, dreamlike atmosphere",
    ];

    (0..count)
        .map(|idx| {
            let i = start_index + idx;
            let width = if i % 3 == 0 {
                512
            } else if i % 3 == 1 {
                768
            } else {
                1024
            };
            let height = if i % 2 == 0 { 768 } else { 512 };
            let model = models[i % models.len()];
            let sampler = samplers[i % samplers.len()];
            let prompt = prompts[i % prompts.len()];

            let meta = ExtractedMetadata {
                format: MetadataFormat::A1111,
                parameters: Some(format!(
                    "{} Steps: 20, Sampler: {}, CFG scale: 7, Seed: {}",
                    prompt,
                    sampler,
                    123456789 + i
                )),
                raw: None,
                prompt: Some(prompt.to_string()),
                negative_prompt: Some("low quality, bad anatomy, worst quality".to_string()),
                steps: Some(20 + (i % 30) as u32),
                sampler: Some(sampler.to_string()),
                cfg_scale: Some(7.0 + ((i % 5) as f64) * 0.5),
                seed: Some((123456789 + (i as u64)).to_string()),
                width: Some(width),
                height: Some(height),
                model_name: Some(model.to_string()),
                model_hash: Some(format!("{:08x}", 0x12345678 + (i % 100))),
            };

            ImageFile {
                id: None,
                folder_id,
                path: format!("F:/benchmark_media/folder_{}/image_{:06}.png", folder_id, i),
                size_bytes: 1_200_000 + ((i % 1_000_000) as u64),
                modified_at: 1_700_000_000 + (i as i64),
                container: Container::Png,
                metadata: Some(meta),
                rating: if i % 5 == 0 {
                    Some(((i % 10) + 1) as u8)
                } else {
                    None
                },
                aesthetic_score: Some(6.0 + ((i % 40) as f64) * 0.1),
                is_favorite: i % 10 == 0,
                is_nsfw: false,
                stack_id: if i % 20 == 0 {
                    Some(format!("stack_{:04}", i / 20))
                } else {
                    None
                },
                stack_order: (i % 20) as i32,
            }
        })
        .collect()
}

fn benchmark_library_queries(size: usize) {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let db_path = temp_dir.path().join("berry.db");
    let db = Database::connect(&db_path).expect("Failed to open db");

    let folder = db
        .add_folder(temp_dir.path().to_string_lossy().as_ref())
        .expect("Failed to create folder");

    println!("\n=======================================================");
    println!(">>> Benchmarking Library Size: {} items", size);
    println!("=======================================================");

    // Ingest data in chunks of 1000
    let ingest_start = Instant::now();
    let chunk_size = 1000;
    for chunk_start in (0..size).step_by(chunk_size) {
        let count = std::cmp::min(chunk_size, size - chunk_start);
        let batch = generate_mock_files(folder.id, chunk_start, count);
        db.upsert_files(&batch).expect("Failed to upsert batch");
    }
    let ingest_elapsed = ingest_start.elapsed();
    println!(
        "Ingested {} rows in {:.2?} ({:.0} rows/s)",
        size,
        ingest_elapsed,
        (size as f64) / ingest_elapsed.as_secs_f64()
    );

    // Warm up the database cache
    let warmup_criteria = SearchCriteria {
        folder_id: Some(folder.id),
        limit: Some(400),
        offset: Some(0),
        sort: Some(FileSortField::ModifiedAt),
        direction: Some(SortDirection::Desc),
        ..Default::default()
    };
    let _ = db
        .search_gallery_files_page(&warmup_criteria)
        .expect("Warmup query failed");

    // 1. Time to first usable gallery page (first 400 items + window total count)
    let iterations = 10;
    let mut page1_latencies = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let t0 = Instant::now();
        let page = db
            .search_gallery_files_page(&warmup_criteria)
            .expect("Query failed");
        let elapsed = t0.elapsed();
        assert_eq!(page.items.len(), std::cmp::min(400, size));
        assert_eq!(page.total, size);
        page1_latencies.push(elapsed);
    }
    page1_latencies.sort();
    let p50 = page1_latencies[iterations / 2];
    let p90 = page1_latencies[(iterations * 9) / 10];
    let avg: std::time::Duration =
        page1_latencies.iter().sum::<std::time::Duration>() / (iterations as u32);
    println!(
        "First usable page (400 items + exact total count): Avg = {:.2?}, P50 = {:.2?}, P90 = {:.2?}",
        avg, p50, p90
    );

    // 2. Deep pagination query latency (at various offsets)
    let test_offsets = if size <= 1_000 {
        vec![0, 400, 600]
    } else if size <= 10_000 {
        vec![0, 2_000, 4_000, 8_000, 9_600]
    } else {
        vec![0, 5_000, 10_000, 25_000, 40_000, 49_600]
    };

    println!(
        "\n--- Deep Pagination: Offset Traversal (LIMIT 400 OFFSET N with COUNT(*) OVER()) ---"
    );
    for &offset in &test_offsets {
        let criteria = SearchCriteria {
            folder_id: Some(folder.id),
            limit: Some(400),
            offset: Some(offset),
            sort: Some(FileSortField::ModifiedAt),
            direction: Some(SortDirection::Desc),
            ..Default::default()
        };
        let mut runs = Vec::with_capacity(5);
        for _ in 0..5 {
            let t0 = Instant::now();
            let page = db
                .search_gallery_files_page(&criteria)
                .expect("Deep query failed");
            runs.push(t0.elapsed());
            assert_eq!(page.total, size);
        }
        runs.sort();
        let median = runs[runs.len() / 2];
        println!(
            "  Offset {:>5} (Page {:>3}): median = {:.2?}",
            offset,
            (offset / 400) + 1,
            median
        );
    }

    // 3. Keyset Cursor vs Offset Comparison (at deep boundary)
    if size >= 10_000 {
        println!("\n--- Keyset Cursor vs Offset at Deep Page ---");
        let deep_offset = if size >= 50_000 { 40_000 } else { 8_000 };

        // Fetch cursor item at deep_offset
        let criteria = SearchCriteria {
            folder_id: Some(folder.id),
            limit: Some(1),
            offset: Some(deep_offset),
            sort: Some(FileSortField::ModifiedAt),
            direction: Some(SortDirection::Desc),
            ..Default::default()
        };
        let anchor_page = db.search_gallery_files_page(&criteria).unwrap();
        let anchor = &anchor_page.items[0];
        let cursor_mtime = anchor.modified_at;
        let cursor_id = anchor.id.unwrap();

        // Keyset query benchmark: WHERE folder_id = ? AND (modified_at < ? OR (modified_at = ? AND id < ?)) LIMIT 400
        let mut keyset_runs = Vec::with_capacity(5);
        for _ in 0..5 {
            let t0 = Instant::now();
            let mut stmt = db
                .connection()
                .prepare_cached(
                    "SELECT id, folder_id, path, container, size_bytes, modified_at, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order
                     FROM files
                     WHERE folder_id = ?1 AND (modified_at < ?2 OR (modified_at = ?2 AND id < ?3))
                     ORDER BY modified_at DESC, id DESC
                     LIMIT 400",
                )
                .unwrap();
            let rows = stmt
                .query_map(
                    rusqlite::params![folder.id, cursor_mtime, cursor_id],
                    |row| Ok(row.get::<_, i64>(0)?),
                )
                .unwrap();
            let ids: Vec<i64> = rows.filter_map(Result::ok).collect();
            keyset_runs.push(t0.elapsed());
            assert!(!ids.is_empty());
        }
        keyset_runs.sort();
        let keyset_median = keyset_runs[keyset_runs.len() / 2];

        // Offset query at deep_offset (without COUNT(*) OVER() to isolate raw traversal)
        let mut raw_offset_runs = Vec::with_capacity(5);
        for _ in 0..5 {
            let t0 = Instant::now();
            let mut stmt = db
                .connection()
                .prepare_cached(
                    "SELECT id, folder_id, path, container, size_bytes, modified_at, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order
                     FROM files
                     WHERE folder_id = ?1
                     ORDER BY modified_at DESC, id DESC
                     LIMIT 400 OFFSET ?2",
                )
                .unwrap();
            let rows = stmt
                .query_map(rusqlite::params![folder.id, deep_offset as i64], |row| {
                    Ok(row.get::<_, i64>(0)?)
                })
                .unwrap();
            let ids: Vec<i64> = rows.filter_map(Result::ok).collect();
            raw_offset_runs.push(t0.elapsed());
            assert!(!ids.is_empty());
        }
        raw_offset_runs.sort();
        let raw_offset_median = raw_offset_runs[raw_offset_runs.len() / 2];

        println!(
            "  At offset {}: Raw OFFSET 400 = {:.2?}, Keyset Cursor 400 = {:.2?} (Speedup: {:.1}x)",
            deep_offset,
            raw_offset_median,
            keyset_median,
            raw_offset_median.as_secs_f64() / keyset_median.as_secs_f64()
        );
    }
}

fn benchmark_thumbnail_manifest_sync(count: usize) {
    println!("\n=======================================================");
    println!(">>> Benchmarking Thumbnail Manifest Sync: {} files", count);
    println!("=======================================================");

    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let cache_dir = temp_dir.path().join("cache");
    let thumb_dir = cache_dir.join("thumbnails");
    fs::create_dir_all(&thumb_dir).expect("Failed to create thumb dir");

    let db_path = temp_dir.path().join("berry.db");
    let _ = Database::connect(&db_path).expect("Failed to init db");

    println!("Creating {} mock thumbnail files on disk...", count);
    let create_start = Instant::now();
    for i in 0..count {
        let file_id = (i + 1) as i64;
        let modified_at = 1_700_000_000 + (i as i64);
        let max_edge = if i % 2 == 0 { 384 } else { 512 };
        let filename = format!("{}_{}_{}.webp", file_id, modified_at, max_edge);
        let file_path = thumb_dir.join(filename);
        let mut file = File::create(file_path).expect("Failed to create thumb file");
        // Write a dummy payload representing a WebP thumbnail header
        let _ = file.write_all(b"RIFF\x20\x00\x00\x00WEBPVP8 ");
    }
    let create_elapsed = create_start.elapsed();
    println!(
        "Created {} files in {:.2?} ({:.0} files/s)",
        count,
        create_elapsed,
        (count as f64) / create_elapsed.as_secs_f64()
    );

    // Benchmark synchronize_thumbnail_manifest
    let sync_start = Instant::now();
    let budget_bytes = 2 * 1024 * 1024 * 1024; // 2 GB
    synchronize_thumbnail_manifest(&cache_dir, &db_path, budget_bytes)
        .expect("Manifest synchronization failed");
    let sync_elapsed = sync_start.elapsed();

    let db = Database::connect(&db_path).expect("Failed to open db");
    let (usage_bytes, manifest_count) = db.thumbnail_cache_usage().expect("Usage check failed");

    println!(
        "Manifest Synchronization Complete:\n  Elapsed Time: {:.2?}\n  Throughput: {:.0} files/s\n  Indexed Entries: {}\n  Indexed Size: {:.2} MB",
        sync_elapsed,
        (count as f64) / sync_elapsed.as_secs_f64(),
        manifest_count,
        (usage_bytes as f64) / (1024.0 * 1024.0)
    );
    assert_eq!(manifest_count, count);
}

fn main() {
    println!("#######################################################");
    println!("# BERRY AI STUDIO - LARGE LIBRARY BENCHMARK SUITE    #");
    println!("#######################################################");

    benchmark_library_queries(1_000);
    benchmark_library_queries(10_000);
    benchmark_library_queries(50_000);

    benchmark_thumbnail_manifest_sync(50_000);

    println!("\nBenchmark suite finished successfully.");
}
