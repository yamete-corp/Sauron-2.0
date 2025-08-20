use memmap2::MmapMut;
use rand;
use rand::Rng;
use regex::bytes::Regex;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use walkdir::WalkDir;
use std::io::Write;
use std::collections::HashSet;
use std::process::Command;
use std::os::windows::process::CommandExt;

fn main() {
    let target_dir = PathBuf::from(env::var("CRATE_OUT_DIR").unwrap());

    // get every single file in our project folder
    let binding = PathBuf::from(env::var("CRATE_MANIFEST_DIR").unwrap());
    let sauron_dir = binding.parent().unwrap();
    let mut file_paths = Vec::new();

    for entry in WalkDir::new(sauron_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let path = entry.path();
            let path_str = path.to_str().unwrap().to_string();
            if
                path_str.contains("target\\release") ||
                path_str.contains("target\\debug") ||
                path_str.contains("post_build_script_manifest")
            {
                continue;
            }
            if let Some(src_index) = path_str.find("src") {
                let src_path = &path_str[src_index..];
                file_paths.push(src_path.to_string());
            }
        }
    }
    let mut unique_file_paths: HashSet<String> = HashSet::new();
    for path in file_paths {
        unique_file_paths.insert(path);
    }

    // Convert the HashSet back to a Vec
    let unique_file_paths: Vec<String> = unique_file_paths.into_iter().collect();
    println!("unique_file_paths: {:#?}", unique_file_paths);
    // Get the executable name from Cargo.toml metadata
    let executable_name = "client.exe";

    // Construct the path to the executable
    let executable_path = target_dir.join(executable_name);
    println!("{:#?}", executable_path);

    println!("Modifying release binary...");

    // Your original code starts here
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&executable_path)
        .expect("Failed to open executable");

    // Create a mutable memory map of the file
    let mut mmap = unsafe { MmapMut::map_mut(&file).expect("Failed to memory map the file") };

    // Define your replacements
    // IMPORTANT: New string must be exactly the same length as the original!
    let mut regex_patterns = vec![
        Regex::new(r"C:\\Users\\[a-zA-Z0-9_]{1,15}.*?\\.rs").unwrap(),
        Regex::new(r"C:\\Users\\[a-zA-Z0-9_]{1,15}\\").unwrap(),
        Regex::new(r"rustc/[a-f0-9]{40}(?:\\[a-zA-Z0-9_]+)+\\.rs").unwrap(),
        Regex::new(r"Documents\\Github").unwrap(),
        Regex::new(r"client.exe").unwrap(),
        Regex::new(r"src/main.rs").unwrap()
    ];

    // for all the files
    for file_path in unique_file_paths {
        let simple_regex_pattern = Regex::new(&format!(r"{}", regex::escape(&file_path))).unwrap();
        regex_patterns.push(simple_regex_pattern);
    }

    // later add more regexes and cleaning, check with strings2.exe for exposed strings

    // Process the binary data
    let data = mmap.as_mut();

    println!("{:#?}", data.len());

    let mut count = 0;

    let mut offsets = Vec::new();

    // Find offsets of strings
    for regex_pattern in regex_patterns {
        for mat in regex_pattern.find_iter(data) {
            let start = mat.start();
            let end = mat.end();
            let length = end - start;
            println!("{}", String::from_utf8_lossy(mat.as_bytes()).into_owned());

            offsets.push((start, end));
        }
    }

    // Replace strings
    for (start, end) in offsets {
        let length = end - start;
        let replacement: Vec<u8> = (0..length)
            .map(|_| {
                // Generate a random printable ASCII character
                let mut rng = rand::thread_rng();
                let char_code = rng.gen_range(32..127); // 32 (space) to 126 (~)
                char_code as u8
            })
            .collect();

        data[start..end].copy_from_slice(&replacement);
        count += 1;
    }

    mmap.flush().expect("Failed to flush changes to disk");
    println!("Removed {} leaked strings", count);

    let output = Command::new(target_dir.join("run.bat"))
        .current_dir(target_dir)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
        .output()
        .unwrap();
    println!("{:#?}", output);
}
