use std::time::Duration;
use std::{ fs, thread };
use std::io::{ BufRead, BufReader, Read };
use std::os::windows::process::CommandExt;
use std::path::{ Path, PathBuf };
use std::process::{ Command, Stdio };
use shared::utils::functions::get_current_exe_dir;
use shared::constants::onion_endpoint;

const PUBLIC_KEY_FILE_BYTES: &[u8] = include_bytes!("./hs_ed25519_public_key");
const SECRET_KEY_FILE_BYTES: &[u8] = include_bytes!("./hs_ed25519_secret_key");

pub struct HiddenServiceRunner {
    pub stdout: String,
    // pub stderr: String,
    pub tor_exe_file_path: PathBuf,
    pub tor_proxy_dir: PathBuf,
    pub torrc_file_path: PathBuf,
}

impl HiddenServiceRunner {
    pub fn new() -> Self {
        let exe_dir = get_current_exe_dir().unwrap();
        let tor_proxy_dir = exe_dir.join("tor_proxy");
        let data_dir = tor_proxy_dir.join("data");
        if !data_dir.join("geoip").exists() || !data_dir.join("geoip6").exists() {
            panic!("Tor Geoip files dont exist");
        }
        let tor_dir = tor_proxy_dir.join("tor");

        let hidden_service_dir = tor_dir.join("hidden_service");
        let hostname_file_path = hidden_service_dir.join("hostname");
        let public_key_file_path = hidden_service_dir.join("hs_ed25519_public_key");
        let secret_key_file_path = hidden_service_dir.join("hs_ed25519_secret_key");

        let torrc_file_path = tor_dir.join("torrc");
        let torrc_data =
            r#"HiddenServiceDir ./tor/hidden_service
HiddenServicePort 80 127.0.0.1:80
HiddenServiceVersion 3
NumEntryGuards 5
CircuitBuildTimeout 60000"#.to_owned();

        Self::verify_and_create_file(&hostname_file_path, onion_endpoint().as_bytes());
        Self::verify_and_create_file(&public_key_file_path, PUBLIC_KEY_FILE_BYTES);
        Self::verify_and_create_file(&secret_key_file_path, SECRET_KEY_FILE_BYTES);
        Self::verify_and_create_file(&torrc_file_path, torrc_data.as_bytes());

        let tor_exe_file_path = tor_dir.join("tor.exe");

        Self {
            tor_proxy_dir,
            tor_exe_file_path,
            torrc_file_path,
            stdout: String::new(),
            // stderr: String::new(),
        }
    }
    fn verify_and_create_file(file_path: &PathBuf, expected_bytes: &[u8]) {
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        if !file_path.exists() || !file_path.is_file() {
            fs::write(file_path, expected_bytes).unwrap();
            println!("file: {:#?}, doesnt exist", file_path)
        } else {
            let actual_bytes = fs::read(file_path).unwrap();
            if actual_bytes != expected_bytes {
                fs::write(file_path, expected_bytes).unwrap();
                println!("file: {:#?}, bytes dont match", file_path)
            }
        }
    }
    pub fn run(&mut self) {
        let mut cmd = Command::new(self.tor_exe_file_path.clone())
            .current_dir(self.tor_proxy_dir.clone()) // Set execution directory
            .raw_arg(format!(r#" -f "{}""#, self.torrc_file_path.to_str().unwrap()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn hidden service process");
        let stdout = cmd.stdout.take().expect("Failed to open stdout");
        let stderr = cmd.stderr.take().expect("Failed to open stderr");

        let mut out_reader = BufReader::new(stdout);
        let mut err_reader = BufReader::new(stderr);

        let mut out_buffer = String::new();

        loop {
            let stdout_bytes = out_reader.read_line(&mut out_buffer).unwrap();

            // let stderr_bytes = err_reader.read_exact(&mut buf).unwrap();

            if stdout_bytes > 0 {
                print!("{}", out_buffer);
                self.stdout.push_str(&out_buffer);
                out_buffer.clear();
            }

            // if stderr_bytes > 0 {
            //     eprint!("{}", String::from_utf8_lossy(&buf[0..stderr_bytes]));
            // }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
