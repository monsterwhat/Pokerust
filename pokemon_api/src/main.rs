use serde::{Deserialize, Serialize};
use warp::{Filter, reject};

mod mysql_client;
use mysql_client::{get_pokemon_from_mysql, Pokemon}; // Import the Pokemon struct

#[derive(Debug)]
struct InvalidParameter;

impl reject::Reject for InvalidParameter {}

// Define a struct for the API response, marked for serialization with Serde
#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

async fn get_pokemon_list() -> Result<impl warp::Reply, warp::Rejection> {
    // Fetch Pokemon from the database
    let pokemon_list = match get_pokemon_from_mysql().await {
        Ok(pokemon_list) => pokemon_list,
        Err(_) => {
            // Handle the error appropriately, e.g., return a 500 Internal Server Error
            return Err(warp::reject::custom(InvalidParameter));
        }
    };

    // Return the Pokemon list as a JSON response
    Ok(warp::reply::json(&pokemon_list))
}

// Entry point of the application
#[tokio::main]
async fn main() {

    // Assume you have a MySQL database running locally with user and password set appropriately.
    let pool = mysql_async::Pool::new("mysql://pokemon:pokemon1234@localhost/pokemon");

    match get_pokemon_from_mysql().await {
        Ok(pokemon_list) => {
            for pokemon in pokemon_list {
                println!("{:?}", pokemon);
            }
        }
        Err(e) => eprintln!("Error fetching Pokemon: {:?}", e),
    }

    // Create a filter for the "/api" path that handles various HTTP methods
    let api_route = warp::path("api").and(
        // GET request handler
        warp::get()
            .and(warp::path("pokemon"))
            .and_then(get_pokemon_list)
            // PUT, POST, DELETE request handlers (dummy implementations for demonstration)
            .or(warp::put().and(warp::path("pokemon")).map(|| warp::reply::json(&ApiResponse {
                message: String::from("Pokemon updated successfully!"),
            })))
            .or(warp::post().and(warp::path("pokemon")).map(|| warp::reply::json(&ApiResponse {
                message: String::from("Pokemon created successfully!"),
            })))
            .or(warp::delete().and(warp::path("pokemon")).map(|| warp::reply::json(&ApiResponse {
                message: String::from("Pokemon deleted successfully!"),
            }))),
    );

    // Start the Warp server, binding it to the address 127.0.0.1:3030
    warp::serve(api_route).run(([127, 0, 0, 1], 3030)).await;
}


