use printavo::Printavo;

#[tokio::main]
async fn main() -> printavo::Result<()> {
    let email = std::env::var("PRINTAVO_EMAIL").expect("PRINTAVO_EMAIL env var is required");
    let token = std::env::var("PRINTAVO_TOKEN").expect("PRINTAVO_TOKEN env var is required");

    let printavo = Printavo::builder().token_auth(email, token).build()?;

    let search_results = printavo
        .orders()
        .search()
        .page(1u32)
        .per_page(10)
        .query("15046")
        .send()
        .await?;

    println!("{:?}", search_results.meta);
    for order in search_results {
        println!("{}", order.order_total)
    }

    Ok(())
}
