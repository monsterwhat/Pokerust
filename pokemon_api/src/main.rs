use serde::{Deserialize, Serialize};
use warp::{Filter, reject};
mod mysql_client;
use mysql_client::{*};
use std::sync::Mutex;

#[derive(Debug)]
struct InvalidParameter;

impl reject::Reject for InvalidParameter {}

// Define a struct for the API response, marked for serialization with Serde
#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

lazy_static::lazy_static! {
    static ref POKEMON_LIST: Mutex<Vec<Pokemon>> = Mutex::new(vec![]);
}

async fn load_list() {
    match get_pokemon_from_mysql().await {
        Ok(pokemon_list) => {
            let mut list = POKEMON_LIST.lock().unwrap();
            list.extend(pokemon_list);
        }
        Err(e) => eprintln!("Error fetching Pokemon: {:?}", e),
    }
}

async fn get_pokemon_list() -> Result<impl warp::Reply, warp::Rejection> {
    // Return the Pokemon list as a JSON response
    let pokemon_list = POKEMON_LIST.lock().unwrap();
    Ok(warp::reply::json(&*pokemon_list))
}

async fn get_pokemon_by_id(id: i32) -> Result<impl warp::Reply, warp::Rejection> {
    // Return the Pokemon by id as a JSON response
    let pokemon_list = POKEMON_LIST.lock().unwrap();
    let pokemon = pokemon_list.iter().find(|p| p.id == id);
    match pokemon {
        Some(p) => Ok(warp::reply::json(&p)),
        None => Err(warp::reject::custom(InvalidParameter)),
    }
}

fn update_pokemon_by_id(id: i32, pokemon: Pokemon) -> Result<impl warp::Reply, warp::Rejection> {
    // Update the Pokemon by id as a JSON response
    let mut pokemon_list = POKEMON_LIST.lock().unwrap();
    let found_pokemon = pokemon_list.iter_mut().find(|p| p.id == id);
    match found_pokemon {
        Some(p) => {
            // Update the Pokemon's name and evolutions
            p.name = pokemon.name.clone();
            p.evolutions = pokemon.evolutions.clone();
            //Should update the DB
            update_pokemon_in_mysql(pokemon);

            Ok(warp::reply::json(&p))
        },
        None => Err(warp::reject::custom(InvalidParameter)),
    }
}

fn delete_pokemon_by_id(id: i32) -> Result<impl warp::Reply, warp::Rejection> {
    // Return the deleted Pokemon as a JSON response
    let mut pokemon_list = POKEMON_LIST.lock().unwrap();
    let pokemon = pokemon_list.iter().find(|p| p.id == id).cloned();
    match pokemon {
        Some(p) => {
            pokemon_list.retain(|p| p.id != id); // Remove the Pokemon from the list
            //Should delete from the DB

            delete_pokemon_from_mysql(id);

            Ok(warp::reply::json(&p))
        },
        None => Err(warp::reject::custom(InvalidParameter)),
    }
}

async fn create_pokemon(id: i32, pokemon: Pokemon) -> Result<impl warp::Reply, warp::Rejection> {
    // Return the created Pokemon as a JSON response
    let mut pokemon_list = POKEMON_LIST.lock().unwrap();
    
    // Update the ID of the provided Pokemon object
    let new_pokemon = Pokemon {
        id,
        name: pokemon.name.clone(), 
        evolutions: pokemon.evolutions.clone(),
    };

    pokemon_list.push(new_pokemon.clone()); // Add the Pokemon to the list
    // Should add to the DB
    create_pokemon_in_mysql(pokemon).await.unwrap();

    Ok(warp::reply::json(&new_pokemon))
}

// Entry point of the application
#[tokio::main]
async fn main() {

    // Load the Pokemon list from the database
    load_list().await;

    // Create a filter for the "/api" path that handles various HTTP methods
let api_route = warp::path("api").and(
    // GET request handler for specific ID
    warp::get()
        .and(warp::path!("pokemon" / i32))
        .and_then(get_pokemon_by_id)
    // GET request handler for all Pokemon
    .or(warp::get()
        .and(warp::path("pokemon"))
        .and_then(get_pokemon_list))
    // PUT, POST, DELETE request handlers under "/pokemon/{id}"
    .or(warp::put()
        .and(warp::path!("pokemon" / i32))
        .and(warp::body::json())
        .and_then(update_pokemon_by_id))
    .or(warp::post()
        .and(warp::path!("pokemon" / i32))
        .and(warp::body::json())
        .and_then(create_pokemon))
    .or(warp::delete()
        .and(warp::path!("pokemon" / i32))
        .and_then(delete_pokemon_by_id)),
);

    // Start the Warp server, binding it to the address 127.0.0.1:3030
    warp::serve(api_route).run(([127, 0, 0, 1], 3030)).await;
}

