use rand;

fn main() {
    // Set a random seed value for obfstr
    let seed = rand::random::<u64>();
    println!("cargo::rustc-env=OBFSTR_SEED={}", seed);
}
