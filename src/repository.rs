use tokio_postgres::{Error, NoTls};

pub async fn hello() -> Result<(), Error> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;

    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");

    Ok(())
}
