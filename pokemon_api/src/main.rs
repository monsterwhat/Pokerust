use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

mod mysql_client;
use mysql_client::{get_pokemon_from_mysql, Pokemon}; // Import the Pokemon struct

// Define a struct for the API response, marked for serialization with Serde
#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

// Define a struct for Pokemon with evolutions
#[derive(Debug, Serialize, Deserialize)]
struct Pokemon {
    name: String,
    evolutions: Vec<String>,
}

async fn get_pokemon_list() -> Result<impl warp::Reply, warp::Rejection> {
    // Fetch Pokemon from the database
    let pokemon_list = match get_pokemon_from_mysql().await {
        Ok(pokemon_list) => pokemon_list,
        Err(_) => {
            // Handle the error appropriately, e.g., return a 500 Internal Server Error
            return Err(warp::reject::server_error());
        }
    };

    // Return the Pokemon list as a JSON response
    Ok(warp::reply::json(&pokemon_list))
}


// Entry point of the application

#[tokio::main]

async fn main() {

    // Create a filter for the "/api" path that handles various HTTP methods

    let api_route = warp::path("api").and(

        // GET request handler

        warp::get()

            .and(warp::path("pokemon"))

            .and_then(||

                // Apply the get_pokemon_list function to handle the GET request

                get_pokemon_list(),

            )

            // PUT request handler (dummy implementation for demonstration)

            .or(warp::put()

                .and(warp::path("pokemon"))

                .map(|| warp::reply::json(&ApiResponse {

                    message: String::from("Pokemon updated successfully!"),

                })))

            // POST request handler (dummy implementation for demonstration)

            .or(warp::post()

                .and(warp::path("pokemon"))

                .map(|| warp::reply::json(&ApiResponse {

                    message: String::from("Pokemon created successfully!"),

                })))

            // DELETE request handler (dummy implementation for demonstration)

            .or(warp::delete()

                .and(warp::path("pokemon"))

                .map(|| warp::reply::json(&ApiResponse {

                    message: String::from("Pokemon deleted successfully!"),

                }))),

    );



    // Start the Warp server, binding it to the address 127.0.0.1:3030

    warp::serve(api_route).run(([127, 0, 0, 1], 3030)).await;



}
