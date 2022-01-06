use std::error::Error;

use openrgb::OpenRGB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // connect to local server
    let client = OpenRGB::connect().await?;

    // get profiles names
    println!("profiles: {:?}", client.get_profiles().await?);

    // save the current configuration to a new profile
    client.save_profile("my profile").await?;

    // load a profile by name
    client.load_profile("my profile").await?;

    // delete a profile by name
    client.delete_profile("my profile").await?;

    Ok(())
}
