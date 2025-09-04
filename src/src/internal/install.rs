use std::fs::File;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use regex;
use crate::internal::*;

pub fn install(pkgs: Vec<String>) {
    let log = File::create("/tmp/axinstall-pacstrap.log").unwrap();
    
    // Create multi-progress to manage multiple progress bars
    let multi_progress = MultiProgress::new();
    
    // Start with unknown length, will be set when we discover total packages
    let download_pb = multi_progress.add(ProgressBar::new_spinner());
    download_pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} [{elapsed_precise}] Downloading packages ({pos} downloaded)")
            .unwrap()
    );
    
    let install_pb = multi_progress.add(ProgressBar::new_spinner());
    install_pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] Installing packages ({pos} installed)")
            .unwrap()
    );
    
    // Shared counters to track totals discovered
    let total_packages = Arc::new(Mutex::new(None::<u64>));

    exec_eval(
        Command::new("pacman")
        .arg("-Sy")
        .status(),
        "Syncing repositories"
    );

    exec_eval(
        Command::new("pacstrap")
            .arg("/mnt")
            .args(&pkgs)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                let mut log = log.try_clone().unwrap();
                let stdout = child.stdout.take().unwrap();
                let stderr = child.stderr.take().unwrap();
                let mut handles = vec![];

                // Handle stdout
                let download_pb_out = download_pb.clone();
                let install_pb_out = install_pb.clone();
                let total_packages_out = total_packages.clone();
                let out_handle = std::thread::spawn({
                    let mut log = log.try_clone().unwrap();
                    move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                writeln!(log, "{}", line).ok();
                                
                                let line_lower = line.to_lowercase();
                                
                                // Look for total package count (e.g., "Packages (203):" or similar)
                                if let Some(captures) = regex::Regex::new(r"packages?\s*\((\d+)\)")
                                    .unwrap()
                                    .captures(&line_lower) 
                                {
                                    if let Ok(total) = captures.get(1).unwrap().as_str().parse::<u64>() {
                                        let mut total_guard = total_packages_out.lock().unwrap();
                                        if total_guard.is_none() {
                                            *total_guard = Some(total);
                                            
                                            // Convert spinners to progress bars with known length
                                            download_pb_out.set_length(total);
                                            download_pb_out.set_style(
                                                ProgressStyle::default_bar()
                                                    .template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan}] {pos}/{len} Downloading packages")
                                                    .unwrap()
                                                    .progress_chars("#>-")
                                            );
                                            
                                            install_pb_out.set_length(total);
                                            install_pb_out.set_style(
                                                ProgressStyle::default_bar()
                                                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green}] {pos}/{len} Installing packages")
                                                    .unwrap()
                                                    .progress_chars("#>-")
                                            );
                                        }
                                    }
                                }
                                
                                // Check for download-related messages
                                if line_lower.contains("downloading") || 
                                   line_lower.contains("retrieving") ||
                                   line_lower.contains("fetching") {
                                    download_pb_out.inc(1);
                                }
                                
                                // Check for install-related messages
                                if line_lower.contains("installing") ||
                                   line_lower.contains("upgrading") {
                                    install_pb_out.inc(1);
                                }
                            }
                        }
                    }
                });
                handles.push(out_handle);

                // Handle stderr
                let download_pb_err = download_pb.clone();
                let install_pb_err = install_pb.clone();
                let total_packages_err = total_packages.clone();
                let err_handle = std::thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            writeln!(log, "{}", line).ok();
                            
                            let line_lower = line.to_lowercase();
                            
                            // Look for total package count
                            if let Some(captures) = regex::Regex::new(r"packages?\s*\((\d+)\)")
                                .unwrap()
                                .captures(&line_lower) 
                            {
                                if let Ok(total) = captures.get(1).unwrap().as_str().parse::<u64>() {
                                    let mut total_guard = total_packages_err.lock().unwrap();
                                    if total_guard.is_none() {
                                        *total_guard = Some(total);
                                        
                                        // Convert spinners to progress bars with known length
                                        download_pb_err.set_length(total);
                                        download_pb_err.set_style(
                                            ProgressStyle::default_bar()
                                                .template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan}] {pos}/{len} Downloading packages")
                                                .unwrap()
                                                .progress_chars("#>-")
                                        );
                                        
                                        install_pb_err.set_length(total);
                                        install_pb_err.set_style(
                                            ProgressStyle::default_bar()
                                                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green}] {pos}/{len} Installing packages")
                                                .unwrap()
                                                .progress_chars("#>-")
                                        );
                                    }
                                }
                            }
                            
                            // Check for download-related messages
                            if line_lower.contains("downloading") || 
                               line_lower.contains("retrieving") ||
                               line_lower.contains("fetching") {
                                download_pb_err.inc(1);
                            }
                            
                            // Check for install-related messages
                            if line_lower.contains("installing") ||
                               line_lower.contains("upgrading") {
                                install_pb_err.inc(1);
                            }
                        }
                    }
                });
                handles.push(err_handle);

                let status = child.wait();
                
                for handle in handles {
                    handle.join().ok();
                }
                
                // Finish progress bars with appropriate messages
                download_pb.finish_with_message("Downloads complete");
                install_pb.finish_with_message("Installation complete");
                
                status
            }),
        format!("Install packages {} (+ dependencies)", pkgs.join(", ")).as_str(),
    );
}