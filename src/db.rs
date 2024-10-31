pub async fn reset_orders(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DROP TABLE IF EXISTS orders")
        .execute(pool)
        .await?;

    sqlx::query!(
        "
        CREATE TABLE orders (
          id INT PRIMARY KEY,
          region_id INT,
          gift_name VARCHAR(50),
          quantity INT
        )"
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn reset_regions(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DROP TABLE IF EXISTS regions")
        .execute(pool)
        .await?;

    sqlx::query!(
        "
        CREATE TABLE regions (
          id INT PRIMARY KEY,
          name VARCHAR(50)
        )"
    )
    .execute(pool)
    .await?;

    Ok(())
}
