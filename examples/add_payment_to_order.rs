use printavo::Printavo;

#[tokio::main]
async fn main() -> printavo::Result<()> {
    let email = std::env::var("PRINTAVO_EMAIL").expect("PRINTAVO_EMAIL env var is required");
    let token = std::env::var("PRINTAVO_TOKEN").expect("PRINTAVO_TOKEN env var is required");

    let printavo = Printavo::builder().token_auth(email, token).build()?;

    let payment = printavo
        .orders()
        .add_payment(1000u32, 100.00, "11/30/2022")
        .book_category_id(3u32)
        .send()
        .await?;

    println!("{:?}", payment);

    Ok(())
}
