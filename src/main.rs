use std::env;
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use tar::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <URL> <OUTPUT_DIR>", args[0]);
        return Ok(());
    }
    let url = &args[1];
    let extract_dir = &args[2];

    // Step 3: Create the extraction directory if it doesn't exist
    if !Path::new(&extract_dir).exists() {
        println!("Creating extraction directory at: {}", extract_dir);
        fs::create_dir_all(&extract_dir)?;
    }

    // Step 4: Stream the tar.gz file directly from the URL
    let client = Client::new();
    let response = client.get(url).send()?;

    // Step 5: Get the content length from the response (if available)
    let total_size = response
        .content_length()
        .ok_or("Could not get content length from the server")?;

    println!("Starting download and extraction...");

    // Step 6: Wrap the response in a BufReader, then in a GzDecoder and pass it to tar::Archive for streaming extraction
    let progress_reader = ProgressReader::new(BufReader::new(response), total_size);
    let tar_gz = GzDecoder::new(progress_reader);
    let mut archive = Archive::new(tar_gz);

    // Step 7: Extract the archive to the specified directory
    archive.unpack(extract_dir)?;

    println!("\nExtraction complete.");
    Ok(())
}

// A custom reader that tracks progress of bytes read
struct ProgressReader<R> {
    inner: R,
    total_size: u64,
    bytes_read: u64,
}

impl<R: BufRead> ProgressReader<R> {
    fn new(inner: R, total_size: u64) -> Self {
        Self {
            inner,
            total_size,
            bytes_read: 0,
        }
    }
}

impl<R: BufRead> BufRead for ProgressReader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        let buf = self.inner.fill_buf()?;
        let buf_len = buf.len() as u64;
        self.bytes_read += buf_len;
        print_progress(self.bytes_read, self.total_size);
        Ok(buf)
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

impl<R: BufRead> std::io::Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let result = self.inner.read(buf)?;
        self.bytes_read += result as u64;
        print_progress(self.bytes_read, self.total_size);
        Ok(result)
    }
}

fn print_progress(bytes_read: u64, total_size: u64) {
    let percentage: f64 = (bytes_read as f64 / total_size as f64) * 100.0;
    print!("\rProgress: {:.2}%", percentage);
    std::io::stdout().flush().unwrap();
}