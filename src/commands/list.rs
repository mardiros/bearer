use super::super::results::{BearerResult};
use super::super::helpers::path;


pub fn command(config_dir: &str) -> BearerResult<()> {
    let clients = path::list_clients(config_dir)?;
    for client in clients {
        println!("{}", client);
    }
    Ok(())
}