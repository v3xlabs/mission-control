use anyhow::Result;

pub mod config;

#[async_std::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let config = config::load_config()?;

    println!("Config: {:?}", config);

    Ok(())
}

