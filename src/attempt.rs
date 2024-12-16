use std::sync::Arc;
use dashmap::DashSet;
use rayon::prelude::*;
use uuid::Uuid;

fn main() {
    let uuids = Arc::new(DashSet::new());
    let maximum_uuids_count = 2u128.pow(122);
    let generate_same_uuids_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let uuid_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let start_time = std::time::Instant::now();

    let uuids_clone = uuids.clone();
    let duplicate_count_clone = generate_same_uuids_count.clone();
    let uuid_count_clone = uuid_count.clone();

    rayon::spawn(move || {
        (0..u64::MAX).into_par_iter().for_each(|_| {
            let new_uuid = Uuid::new_v4();

            if !uuids_clone.insert(new_uuid.to_string()) {
                duplicate_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }

            uuid_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        });
    });

    let mut last_time = start_time;
    let mut last_count = 0;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let now = std::time::Instant::now();
        let elapsed_secs = (now - last_time).as_secs();

        let total_generated = uuid_count.load(std::sync::atomic::Ordering::Relaxed);
        let total_duplicates = generate_same_uuids_count.load(std::sync::atomic::Ordering::Relaxed);

        let generated_since_last = total_generated - last_count;
        last_count = total_generated;

        println!(
            "Time: {}s | Total Generated: {} ({}%) | Duplicates: {} ({}%) | Rate: {} UUIDs/s",
            start_time.elapsed().as_secs(),
            total_generated,
            (total_generated as f64 * 100_f64) / maximum_uuids_count as f64,
            total_duplicates,
            (total_duplicates * 100) / total_generated,
            generated_since_last / elapsed_secs as usize
        );

        last_time = now;
    }
}
