use std::error::Error;

mod common;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	common::main_prime().await
}
