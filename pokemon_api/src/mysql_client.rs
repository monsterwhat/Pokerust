use mysql_async::prelude::Queryable;
use mysql_async::{Pool, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pokemon {
    pub id: i32,
    pub name: String,
    pub evolutions: String, // Assuming evolutions is stored as a string for simplicity
}

// Define the get_pokemon_from_mysql function, marked as public
pub async fn get_pokemon_from_mysql() -> Result<Vec<Pokemon>, mysql_async::Error> {
    // Assume you have a MySQL database running locally with user and password set appropriately.
    let pool = Pool::new("mysql://pokemon:pokemon1234@localhost:3306/pokemon");
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

// Define the create_pokemon_in_mysql function, marked as public
pub async fn create_pokemon_in_mysql(pokemon: Pokemon) -> Result<(), mysql_async::Error> {
    let pool = Pool::new("mysql://pokemon:pokemon1234@localhost:3306/pokemon");
    let mut conn = pool.get_conn().await?;

    let query = "INSERT INTO pokemon (name, evolutions) VALUES (?, ?)";
    conn.exec_drop(query, (&pokemon.name, &pokemon.evolutions)).await?;

    Ok(())
}

// Define the update_pokemon_in_mysql function, marked as public
pub async fn update_pokemon_in_mysql(pokemon: Pokemon) -> Result<(), mysql_async::Error> {
    
    let pool = Pool::new("mysql://pokemon:pokemon1234@localhost:3306/pokemon");
    let mut conn = pool.get_conn().await?;

    let query = format!(
        "UPDATE pokemon SET name = '{}', evolutions = '{}' WHERE id = {}",
        pokemon.name, pokemon.evolutions, pokemon.id
    );

    conn.query_drop(&query).await?;

    Ok(())
}

// Define the delete_pokemon_from_mysql function, marked as public
pub async fn delete_pokemon_from_mysql(id: i32) -> Result<(), mysql_async::Error> {
    
    let pool = Pool::new("mysql://pokemon:pokemon1234@localhost:3306/pokemon");
    let mut conn = pool.get_conn().await?;

    let query = format!("DELETE FROM pokemon WHERE id = {}", id);

    conn.query_drop(&query).await?;

    Ok(())
}





