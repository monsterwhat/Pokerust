use mysql_async::{Pool, Row};
use mysql_async::prelude::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Pokemon {
    id: i32,
    name: String,
    evolutions: String, // Assuming evolutions is stored as a string for simplicity
}

// Define the get_pokemon_from_mysql function, marked as public
pub async fn get_pokemon_from_mysql() -> Result<Vec<Pokemon>, mysql_async::Error> {
    // Assume you have a MySQL database running locally with user and password set appropriately.
    let pool = Pool::new("mysql://pokemon:pokemon1234@localhost/pokemon");
    let mut conn = pool.get_conn().await?;

    let query = "SELECT id, name, evolutions FROM pokemon";
    let rows: Vec<Row> = conn.query(query).await?;

    let pokemon_list: Vec<Pokemon> = rows
        .into_iter()
        .map(|row| {
            let id: i32 = row.get("id").unwrap_or_default();
            let name: String = row.get("name").unwrap_or_default();
            let evolutions: String = row.get("evolutions").unwrap_or_default();

            Pokemon { id, name, evolutions }
        })
        .collect();

    Ok(pokemon_list)
}

