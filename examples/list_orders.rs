use printavo::{params::Direction, Printavo};
use time::macros::datetime;

#[tokio::main]
async fn main() -> printavo::Result<()> {
    let email = std::env::var("PRINTAVO_EMAIL").expect("PRINTAVO_EMAIL env var is required");
    let token = std::env::var("PRINTAVO_TOKEN").expect("PRINTAVO_TOKEN env var is required");

    let printavo = Printavo::builder().token_auth(email, token).build()?;

    let orders = printavo
        .orders()
        .list()
        .page(2u32)
        .per_page(10)
        .sort_column("id")
        .direction(Direction::Ascending)
        .in_production_after(datetime!(2022-11-01 0:00 UTC))
        .in_production_before(datetime!(2022-11-30 0:00 UTC))
        .send()
        .await?;

    println!("{:?}", orders.meta);
    for order in orders {
        println!("{}", order.id)
    }

    Ok(())
}
