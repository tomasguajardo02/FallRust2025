mod executor;
use executor::TaskExecutor;

use std::{
    collections::HashMap,
    time::{Duration, Instant},
    sync::{Arc, Mutex},
    path::PathBuf,
};
use std::fs::{self, File};
use std::io::Read;
use walkdir::WalkDir;
use encoding_rs::{Encoding, UTF_8};
use crossbeam_channel::{self, Sender, Receiver};


/// ----------------------------------------------------
/// 1. DATA STRUCTURES
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

// Struct for channel communication (warnings suppressed via underscores)
struct StatusUpdate {
    pub name: String,
    pub _start_time: Instant,
    pub is_error: bool,
    pub _final_analysis: FileAnalysis, 
}


/// ----------------------------------------------------
/// 2. ANALYSIS LOGIC AND HELPERS
/// ----------------------------------------------------

const DEFAULT_ENCODING: &'static Encoding = UTF_8; 

fn decode_file_content(path: &PathBuf) -> Result<String, ProcessingError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(ProcessingError::Filesystem(format!("File open failed: {}", e))),
    };

    let mut bytes = Vec::new();
    
    if let Err(e) = file.read_to_end(&mut bytes) {
        return Err(ProcessingError::Filesystem(format!("File read failed: {}", e)));
    }

    let (content, _encoding_used, had_errors) = DEFAULT_ENCODING.decode(&bytes);

    if had_errors {
        return Err(ProcessingError::EncodingError(format!(
            "Content encoding failed."
        )));
    }
    
    Ok(content.into_owned())
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

fn discover_files(directory_paths: &[&str]) -> Vec<PathBuf> {
    let mut discovered = Vec::new();
    for dir in directory_paths {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if path.is_file() && path.file_name().map_or(false, |name| 
                name != "target" && name != "Cargo.toml" && name != "Cargo.lock") {
                discovered.push(path);
            }
        }
    }
    discovered
}

fn print_final_report(final_results: &Vec<FileAnalysis>) {
    println!("\n--- EXECUTION COMPLETE ---");
    
    let total = final_results.len();
    let successful = final_results.iter().filter(|r| r.errors.is_empty()).count();
    let total_time: Duration = final_results.iter().map(|r| r.processing_time).sum();
    let total_errs = final_results.iter().map(|r| r.errors.len()).sum::<usize>();

    println!("Total Items Scanned: {}", total);
    println!("Successful Analysis: {}", successful);
    println!("Total Errors Logged: {}", total_errs);
    println!("Cumulative Task Time: {:?}", total_time);
}


/// ----------------------------------------------------
/// 3. MAIN CONTROLLER AND EXECUTION
/// ----------------------------------------------------

fn main() {
    let sources = vec!["."]; 
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

    let executor = TaskExecutor::new(thread_capacity); 

    let (status_tx, status_rx): (Sender<StatusUpdate>, Receiver<StatusUpdate>) = 
        crossbeam_channel::unbounded();

    for path in files_to_process {
        if *system_cancellation.lock().unwrap() {
            eprintln!("System cancellation activated. Halting submission.");
            break;
        }

        let path_clone = path.clone();
        let store_clone = Arc::clone(&result_store);
        let status_tx_clone = status_tx.clone();
        
        let path_string = path.to_string_lossy().to_string();
        
        // Clone the string once for use within the worker thread
        let name_for_worker = path_string.clone(); 

        progress_state.active_tasks.insert(path_string, Instant::now());

        executor.submit(move || {
            let start_time = Instant::now();
            let result = perform_analysis(&path_clone);
            
            let (is_error, final_analysis_name, final_analysis) = match result {
                Ok(analysis) => {
                    // Success: Use the name from the analysis struct
                    let name_clone = analysis.filename.clone();
                    (false, name_clone, analysis)
                }
                Err(e) => {
                    // Error: Use the name captured by the worker thread (name_for_worker)
                    let error_analysis = FileAnalysis {
                        filename: name_for_worker.clone(),
                        stats: FileStats {
                            word_count: 0, line_count: 0, char_frequencies: HashMap::new(), size_bytes: 0,
                        },
                        errors: vec![e],
                        processing_time: start_time.elapsed(),
                    };
                    // Move the final name string here for use in the StatusUpdate struct
                    (true, name_for_worker, error_analysis) 
                }
            };

            // 1. Store the final result
            store_clone.lock().unwrap().push(final_analysis); 

            // 2. Send the status update via the channel
            if let Err(e) = status_tx_clone.send(StatusUpdate {
                name: final_analysis_name, // Use the name available from the match arm
                _start_time: start_time,
                is_error,
                _final_analysis: FileAnalysis {
                    filename: String::new(), stats: FileStats{word_count: 0, line_count: 0, char_frequencies: HashMap::new(), size_bytes: 0}, 
                    errors: Vec::new(), processing_time: start_time.elapsed(),
                },
            }) {
                eprintln!("Failed to send status update: {}", e);
            }
        });
    }

    drop(status_tx);
    drop(executor); 
    
    
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
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // Keep looping if channel is quiet
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
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
    
    print_final_report(&result_store.lock().unwrap());
}