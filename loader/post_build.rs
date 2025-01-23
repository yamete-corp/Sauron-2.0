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
    // get every single file in our project src folder
    let binding = PathBuf::from(env::var("CRATE_MANIFEST_DIR").unwrap());
    let sauron_dir = binding.parent().unwrap();

    let strings2_dir = sauron_dir.join("strings2.exe");
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
    let executable_name = "loader.exe";

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
        Regex::new(r"C:\\Users\\[a-zA-Z0-9_]{1,15}.*?\.rs").unwrap(),
        Regex::new(r"C:\\Users\\[a-zA-Z0-9_]{1,15}\\").unwrap(),
        Regex::new(r"rustc/[a-f0-9]{40}(?:\\[a-zA-Z0-9_]+)+\.rs").unwrap(),
        Regex::new(r"loader.pdb").unwrap(),
        Regex::new(r"loader.exe").unwrap(),
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

    // Create a new padded binary file
    let padded_executable_name = "io-utils.js.map";
    let padded_executable_path = target_dir.join(padded_executable_name);
    let mut padded_file = fs::File
        ::create(&padded_executable_path)
        .expect("Failed to create padded executable file");

    // Generate 1KB of random padding
    let mut rng = rand::thread_rng();
    let padding: Vec<u8> = (0..1024).map(|_| rng.gen::<u8>()).collect();

    // Write 1KB of padding to the beginning of the file
    padded_file.write_all(&padding).expect("Failed to write padding to file");

    // Write the original executable data to the file
    // let original_file = fs::File
    //     ::open(&executable_path)
    //     .expect("Failed to open original executable file");
    let mut original_data = fs
        ::read(executable_path.clone())
        .expect("Failed to read original executable data");
    padded_file.write_all(&original_data).expect("Failed to write original data to file");

    // Write 10KB of padding to the end of the file
    padded_file.write_all(&padding).expect("Failed to write padding to file");

    println!("Created padded executable file: {}", padded_executable_path.display());

    let basic_dir = target_dir.parent().unwrap().parent().unwrap();

    let output = Command::new(basic_dir.join("strings.bat"))
        .current_dir(target_dir)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
        .output()
        .unwrap();
    println!("{:#?}", output);
}
