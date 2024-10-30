use tokio::{fs, task};
use rand::{thread_rng, Rng};
use std::path::Path;
use std::time::Instant;
use num_cpus;

async fn generate_random_data(size: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    (0..size).map(|_| rng.gen::<u8>()).collect()
}

async fn write_files_to_dir(dir: &str, files_count: usize, min_size: usize, max_size: usize) 
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 确保目录存在
    fs::create_dir_all(dir).await?;
    
    let mut handles = vec![];
    
    for i in 0..files_count {
        let dir = dir.to_string();
        let handle = task::spawn(async move {
            let size = {
                let mut rng = thread_rng();
                rng.gen_range(min_size..=max_size)
            };
            let data = generate_random_data(size).await;
            let file_path = Path::new(&dir).join(format!("file_{}.dat", i));
            
            fs::write(file_path, data).await
        });
        handles.push(handle);
    }

    // 等待所有文件写入完成
    for handle in handles {
        handle.await??;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let base_dir = args.get(1).map(|s| s.as_str()).unwrap();
    
    let thread_count = num_cpus::get(); // 使用系统CPU核心数
    let files_per_thread = 10000; // 每个线程写入的文件数
    let min_size = 30 * 1024; // 最小文件大小 (10KB)
    let max_size = 30 * 1024; // 最大文件大小 (50KB)

    let start = Instant::now();
    let mut thread_handles = vec![];

    // 创建多个线程，每个线程负责一个目录
    for thread_id in 0..thread_count {
        let dir = format!("{}/thread_{}", base_dir, thread_id);
        fs::create_dir_all(&dir).await?;
        let handle = task::spawn(async move {
            write_files_to_dir(&dir, files_per_thread, min_size, max_size).await
        });
        thread_handles.push(handle);
    }

    // 等待所有线程完成
    for handle in thread_handles {
        handle.await??;
    }

    let duration = start.elapsed();
    println!("总耗时: {:?}", duration);
    println!("总文件数: {}", thread_count * files_per_thread);
    println!("平均每秒写入文件数: {:.2}", 
        (thread_count * files_per_thread) as f64 / duration.as_secs_f64());

    Ok(())
}
