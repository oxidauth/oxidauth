use oxidauth_kernel::error::BoxedError;

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    // println!("testing client...");

    // let client = OxidAuthClient::new(
    //     &"http://api.oxidauth.localhost".parse()?,
    //     "malreynolds",
    //     "password123",
    // )?;

    // // client
    // //     .get(
    // //         "/users/by_username/malreynolds",
    // //         None::<()>,
    // //     )
    // //     .await?;

    // client.authenticate().await?;
    // println!("authenticated");

    // client.refresh().await?;
    // println!("refreshed");

    // println!("WOOT!!");

    Ok(())
}
