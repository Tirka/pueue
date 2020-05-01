use ::anyhow::{bail, Result};
use ::log::error;
use ::snap::write::FrameEncoder;
use ::std::fs::{remove_file, File};
use ::std::io;
use ::std::io::prelude::*;
use ::std::path::{Path, PathBuf};

use crate::settings::Settings;

/// Return the paths to temporary stdout and stderr files for a task
pub fn get_log_paths(task_id: usize, settings: &Settings) -> (PathBuf, PathBuf) {
    let pueue_dir = Path::new(&settings.daemon.pueue_directory).join("task_logs");
    let out_path = pueue_dir.join(format!("{}_stdout.log", task_id));
    let err_path = pueue_dir.join(format!("{}_stderr.log", task_id));
    (out_path, err_path)
}

/// Create and return the file handle for temporary stdout and stderr files for a task
pub fn create_log_file_handles(task_id: usize, settings: &Settings) -> Result<(File, File)> {
    let (out_path, err_path) = get_log_paths(task_id, settings);
    let stdout = File::create(out_path)?;
    let stderr = File::create(err_path)?;

    Ok((stdout, stderr))
}

/// Return the file handle for temporary stdout and stderr files for a task
pub fn get_log_file_handles(task_id: usize, settings: &Settings) -> Result<(File, File)> {
    let (out_path, err_path) = get_log_paths(task_id, settings);
    let stdout = File::open(out_path)?;
    let stderr = File::open(err_path)?;

    Ok((stdout, stderr))
}

/// Return the content of temporary stdout and stderr files for a task
pub fn read_log_files(task_id: usize, settings: &Settings) -> Result<(String, String)> {
    let (mut stdout_handle, mut stderr_handle) = get_log_file_handles(task_id, settings)?;
    let mut stdout_buffer = Vec::new();
    let mut stderr_buffer = Vec::new();

    stdout_handle.read_to_end(&mut stdout_buffer)?;
    stderr_handle.read_to_end(&mut stderr_buffer)?;

    let stdout = String::from_utf8_lossy(&stdout_buffer);
    let stderr = String::from_utf8_lossy(&stderr_buffer);

    Ok((stdout.to_string(), stderr.to_string()))
}

/// Remove temporary stdout and stderr files for a task
pub fn clean_log_handles(task_id: usize, settings: &Settings) {
    let (out_path, err_path) = get_log_paths(task_id, settings);
    if let Err(err) = remove_file(out_path) {
        error!(
            "Failed to remove stdout file for task {} with error {:?}",
            task_id, err
        );
    };
    if let Err(err) = remove_file(err_path) {
        error!(
            "Failed to remove stderr file for task {} with error {:?}",
            task_id, err
        );
    };
}

/// Return stdout and stderr of a finished process
/// Everything is compressed using Brotli and then encoded to Base64
pub fn read_and_compress_log_files(
    task_id: usize,
    settings: &Settings,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let (mut stdout_handle, mut stderr_handle) = match get_log_file_handles(task_id, settings) {
        Ok((stdout, stderr)) => (stdout, stderr),
        Err(err) => {
            bail!("Error while opening the output files: {}", err);
        }
    };

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    {
        // Compress log input and pipe it into the base64 encoder
        let mut stdout_compressor = FrameEncoder::new(&mut stdout);
        io::copy(&mut stdout_handle, &mut stdout_compressor)?;
        let mut stderr_compressor = FrameEncoder::new(&mut stderr);
        io::copy(&mut stderr_handle, &mut stderr_compressor)?;
    }

    Ok((stdout, stderr))
}
