use lazy_static::lazy_static;
use shared::logs::logger::Logger;
use std::{ env, path::Path, sync::RwLock };
use obfstr::obfstr as s;

lazy_static! {
    static ref LOGGER: RwLock<Option<Logger>> = RwLock::new(None);
}
fn main() {
    let mut args: Vec<String> = env::args().collect();

    // first arg is always the executable path
    args.remove(0);

    // check where we at:
    let appdata_dir = env::var(s!("APPDATA")).unwrap_or(s!("%APPDATA%").to_owned());
    let log_directory = Path::new(&appdata_dir).join(s!("SharedLocal"));

    if let Ok(logger) = Logger::new(log_directory.clone(), "ss".to_owned()) {
        let mut lock = LOGGER.write().unwrap();
        *lock = Some(logger);
    }
}
