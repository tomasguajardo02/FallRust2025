mod executor;
use executor::TaskExecutor;

use std::{
    collections::HashMap,
    time::{Duration, Instant},
    // Standard library mpsc channel (multiple producer, single consumer)
    sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}}, 
    path::PathBuf,
};
use std::fs::{self, File};
use std::io::{self, Read}; 

/// ----------------------------------------------------
/// 1. DATA STRUCTURES (REQUIRED OUTPUT FORMAT)
/// ----------------------------------------------------

pub struct FileAnalysis {
    pub filename: String,
    pub stats: FileStats,
    pub errors: Vec<ProcessingError>,
    pub processing_time: Duration,
}

pub struct FileStats {
    pub word_count: usize,
    pub line_count: usize,
    pub char_frequencies: HashMap<char, usize>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub enum ProcessingError {
    Filesystem(String),
    AnalysisIssue(String),
    Cancellation(String),
    EncodingError(String), 
}

pub struct TrackingReport {
    pub total_tasks: usize,
    pub finished_count: usize,
    pub active_tasks: HashMap<String, Instant>, 
    pub failure_count: usize,
}

// Struct for channel communication (Standard library mpsc)
struct StatusUpdate {
    pub name: String,
    pub is_error: bool,
}


/// ----------------------------------------------------
/// 2. ANALYSIS LOGIC AND HELPERS
/// ----------------------------------------------------

// Uses standard library File::read_to_string
fn decode_file_content(path: &PathBuf) -> Result<String, ProcessingError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(ProcessingError::Filesystem(format!("File open failed: {}", e))),
    };

    let mut content = String::new();
    
    // Reads directly into String, handling UTF-8 decoding
    if let Err(e) = file.read_to_string(&mut content) {
        return Err(ProcessingError::EncodingError(format!(
            "Content read/decoding failed: {}", e
        )));
    }
    
    Ok(content)
}

fn perform_analysis(path: &PathBuf) -> Result<FileAnalysis, ProcessingError> {
    let timer = Instant::now();
    let name = path.to_string_lossy().to_string();
    let mut failure_list = Vec::new();
    
    let size_bytes = match fs::metadata(path) {
        Ok(meta) => meta.len(),
        Err(e) => return Err(ProcessingError::Filesystem(format!("Metadata access failed: {}", e))),
    };

    let content = match decode_file_content(path) {
        Ok(c) => c,
        Err(e) => {
            failure_list.push(e);
            "".to_string() 
        }
    };

    let line_count = content.lines().count();
    let word_count = content.split_whitespace().count(); 

    let mut char_frequencies = HashMap::new();
    for ch in content.chars() {
        *char_frequencies.entry(ch).or_insert(0) += 1;
    }

    let stats = FileStats {
        word_count,
        line_count,
        char_frequencies,
        size_bytes,
    };
    
    Ok(FileAnalysis {
        filename: name,
        stats,
        errors: failure_list,
        processing_time: timer.elapsed(),
    })
}

// Standard library recursive directory traversal 
fn recursive_discover(dir_path: &PathBuf, discovered: &mut Vec<PathBuf>) -> io::Result<()> {
    let banned_names = ["target", "Cargo.toml", "Cargo.lock"];

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
             if banned_names.contains(&file_name) {
                continue;
            }
        }
        
        if path.is_dir() {
            // Recursively search directories
            recursive_discover(&path, discovered)?;
        } else if path.is_file() {
            discovered.push(path);
        }
    }
    Ok(())
}

fn discover_files(directory_paths: &[&str]) -> Vec<PathBuf> {
    let mut discovered = Vec::new();
    for dir in directory_paths {
        if let Err(e) = recursive_discover(&PathBuf::from(dir), &mut discovered) {
            eprintln!("Error discovering files in {}: {}", dir, e);
        }
    }
    discovered
}

// Calculates and prints cumulative statistics
fn print_final_report(final_results: &Vec<FileAnalysis>) {
    let total = final_results.len();
    let successful = final_results.iter().filter(|r| r.errors.is_empty()).count();
    let total_errs = final_results.iter().map(|r| r.errors.len()).sum::<usize>();

    // Calculate Cumulative Statistics
    let total_time: Duration = final_results.iter().map(|r| r.processing_time).sum();
    let total_word_count: usize = final_results.iter().map(|r| r.stats.word_count).sum();
    let total_line_count: usize = final_results.iter().map(|r| r.stats.line_count).sum();
    let total_size_bytes: u64 = final_results.iter().map(|r| r.stats.size_bytes).sum();
    
    fn format_bytes(bytes: u64) -> String {
        const MB: f64 = 1024.0 * 1024.0;
        format!("{:.2} MB", bytes as f64 / MB)
    }

    println!("\n--- EXECUTION COMPLETE ---");
    // Standard required summary
    println!("Total Items Scanned: {}", total);
    println!("Successful Analysis: {}", successful);
    println!("Total Errors Logged: {}", total_errs);
    
    // Cumulative Statistics Summary 
    println!("\n==============================================");
    println!("Cumulative Analysis Results:");
    println!("Total Books Processed: {}", successful); 
    println!("Total Line Count: {}", total_line_count);
    println!("Total Word Count: {}", total_word_count);
    println!("Total Data Volume: {}", format_bytes(total_size_bytes));
    println!("Total Processing Time: {:?}", total_time);
    println!("==============================================");
}


/// ----------------------------------------------------
/// 3. MAIN CONTROLLER AND EXECUTION
/// ----------------------------------------------------

fn main() {
    // CRITICAL: This MUST point to the folder containing your 100+ .txt books.
    let sources = vec!["./gutenberg_books"]; 
    let thread_capacity = 8; 

    let result_store: Arc<Mutex<Vec<FileAnalysis>>> = Arc::new(Mutex::new(Vec::new()));
    
    let mut progress_state = TrackingReport {
        total_tasks: 0, finished_count: 0, active_tasks: HashMap::new(), failure_count: 0,
    };
    
    let system_cancellation = Arc::new(Mutex::new(false)); 

    let files_to_process: Vec<PathBuf> = discover_files(&sources); 
    
    progress_state.total_tasks = files_to_process.len();
    let total_tasks = progress_state.total_tasks;
    println!("Total files scheduled for processing: {}", total_tasks);
    
    // Your custom thread pool 
    let executor = TaskExecutor::new(thread_capacity); 

    // Standard library mpsc channel setup
    let (status_tx, status_rx): (Sender<StatusUpdate>, Receiver<StatusUpdate>) = 
        mpsc::channel();

    for path in files_to_process {
        if *system_cancellation.lock().unwrap() {
            eprintln!("System cancellation activated. Halting submission.");
            break;
        }

        let path_clone = path.clone();
        let store_clone = Arc::clone(&result_store);
        let status_tx_clone = status_tx.clone();
        
        let path_string = path.to_string_lossy().to_string();
        let name_for_worker = path_string.clone(); 

        progress_state.active_tasks.insert(path_string, Instant::now());

        executor.submit(move || {
            let start_time = Instant::now();
            let result = perform_analysis(&path_clone);
            
            let (is_error, final_analysis_name, final_analysis) = match result {
                Ok(analysis) => {
                    let name_clone = analysis.filename.clone();
                    (false, name_clone, analysis)
                }
                Err(e) => {
                    let error_analysis = FileAnalysis {
                        filename: name_for_worker.clone(),
                        stats: FileStats {
                            word_count: 0, line_count: 0, char_frequencies: HashMap::new(), size_bytes: 0,
                        },
                        errors: vec![e],
                        processing_time: start_time.elapsed(),
                    };
                    (true, name_for_worker, error_analysis) 
                }
            };

            store_clone.lock().unwrap().push(final_analysis); 

            if let Err(e) = status_tx_clone.send(StatusUpdate {
                name: final_analysis_name, 
                is_error,
            }) {
                eprintln!("Failed to send status update: {}", e);
            }
        });
    }

    drop(status_tx);
    
    
    // --- MAIN THREAD PROGRESS RECEIVER LOOP ---
    println!("\n--- Real-time Progress ---");
    
    let mut progress = progress_state;
    
    while progress.finished_count < total_tasks {
        match status_rx.recv_timeout(Duration::from_millis(100)) { 
            Ok(update) => {
                progress.active_tasks.remove(&update.name);
                progress.finished_count += 1;
                if update.is_error {
                    progress.failure_count += 1;
                }
                
                let completed = progress.finished_count;
                let errors = progress.failure_count;
                let active = progress.active_tasks.len();
                
                println!(
                    "REPORT: Done: {}/{} | Working: {} | Failures: {}",
                    completed, total_tasks, active, errors
                );
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Keep looping if channel is quiet
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Process final updates and exit loop
                while let Ok(update) = status_rx.try_recv() {
                    progress.active_tasks.remove(&update.name);
                    progress.finished_count += 1;
                    if update.is_error {
                        progress.failure_count += 1;
                    }
                }
                break;
            }
        }
    }
    
    drop(executor); 

    print_final_report(&result_store.lock().unwrap());
}