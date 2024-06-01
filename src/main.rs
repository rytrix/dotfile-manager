mod manager;

use manager::Manager;

fn main() -> std::io::Result<()> {
    let text = std::fs::read_to_string("tests/test.toml")?;

    let manager = Manager::new(text.as_str(), true)?;
    manager.list_entries();
    println!();
    manager.list_full_config();
    println!();
    manager.deploy_config("debian");

    Ok(())
}
