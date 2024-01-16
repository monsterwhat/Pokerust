use mysql_async::{Pool, Row, Params, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Pokemon {
    id: i32,
    name: String,
    evolutions: String, // Assuming evolutions is stored as a string for simplicity
}

async fn get_pokemon_from_mysql(pool: &Pool) -> Result<Vec<Pokemon>, mysql_async::Error> {
    let mut conn = pool.get_conn().await?;

    let query = "SELECT id, name, evolutions FROM pokemon";
    let rows: Vec<Row> = conn.query(query, Params::Empty).await?;

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

#[tokio::main]
async fn main() {
    // Assume you have a MySQL database running locally with user and password set appropriately.
    let pool = mysql_async::Pool::new("mysql://pokemon:pokemon1234@localhost/pokemon").await.unwrap();

    match get_pokemon_from_mysql(&pool).await {
        Ok(pokemon_list) => {
            for pokemon in pokemon_list {
                println!("{:?}", pokemon);
            }
        }
        Err(e) => eprintln!("Error fetching Pokemon: {:?}", e),
    }
}
