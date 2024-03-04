use warp::{Filter, reject};
mod mysql_client;
use mysql_client::{*};
use std::sync::{Mutex, Arc};

#[derive(Debug)]
struct InvalidParameter;

impl reject::Reject for InvalidParameter {}

lazy_static::lazy_static! {
    static ref POKEMON_LIST: Arc<Mutex<Vec<Pokemon>>> = Arc::new(Mutex::new(vec![]));
}

async fn load_list() {
    match get_pokemon_from_mysql().await {
        Ok(pokemon_list) => {
            match POKEMON_LIST.lock() {
                Ok(mut list) => {
                    list.extend(pokemon_list);
                    //Display a success message for debugging purposes
                    println!("Pokemon list loaded/reloaded successfully");
                }
                Err(_) => {
                    eprintln!("Failed to lock POKEMON_LIST");
                }
            }
        }
        Err(e) => eprintln!("Error fetching Pokemon: {:?}", e),
    }
}

async fn get_pokemon_list() -> Result<impl warp::Reply, warp::Rejection> {
    // Return the Pokemon list as a JSON response
    let pokemon_list = POKEMON_LIST.lock().map_err(|_| warp::reject::custom(InvalidParameter))?;
    println!("Pokemon list served successfully");
    Ok(warp::reply::json(&*pokemon_list))
    
}

async fn get_pokemon_by_id(id: i32) -> Result<impl warp::Reply, warp::Rejection> {
    // Return the Pokemon by id as a JSON response
    let pokemon_list = POKEMON_LIST.lock().map_err(|_| warp::reject::custom(InvalidParameter))?;
    let pokemon = pokemon_list.iter().find(|p| p.id == id);

    match pokemon {
        Some(p) => 
        Ok(warp::reply::json(&p)),
        None => 
        Err(warp::reject::custom(InvalidParameter)),
    }
}

async fn update_pokemon_by_id(id: i32, pokemon: Pokemon) -> Result<impl warp::Reply, warp::Rejection> {
    let found_pokemon = {
        let mut pokemon_list = POKEMON_LIST.lock().map_err(|_| warp::reject::custom(InvalidParameter))?;
        pokemon_list.iter_mut().find(|p| p.id == id).cloned()
    };

    match found_pokemon {
        Some(mut p) => {
            // Update the Pokemon object with the provided values
            p.name = pokemon.name;
            p.evolutions = pokemon.evolutions;

            {
                //find the pokemon on the list and update it remember this is inside a mutexGuard
                let mut pokemon_list = POKEMON_LIST.lock().map_err(|_| warp::reject::custom(InvalidParameter))?;
                let found_pokemon_index = pokemon_list.iter().position(|p| p.id == id).unwrap();
                pokemon_list[found_pokemon_index] = p.clone(); 
            }
            // Update the DB
            update_pokemon_in_mysql(p.clone()).await.map_err(|_| warp::reject::custom(InvalidParameter))?;

            //Print to console the change for debugging purposes
            println!("Pokemon with name {} updated", p.name);
            load_list().await;

            Ok(warp::reply::json(&p))
        },
        None => Err(warp::reject::custom(InvalidParameter)),
    }
}

async fn delete_pokemon_by_id(id: i32) -> Result<impl warp::Reply, warp::Rejection> {
    let found_pokemon_index = {
        let pokemon_list = POKEMON_LIST.lock().map_err(|_| warp::reject::custom(InvalidParameter))?;
        pokemon_list.iter().position(|p| p.id == id)
    };

    match found_pokemon_index {
        Some(index) => {
            //Delete the Pokemon from DB
            delete_pokemon_from_mysql(id).await.map_err(|_| warp::reject::custom(InvalidParameter))?;
            
            {
                let mut pokemon_list = POKEMON_LIST.lock().map_err(|_| warp::reject::custom(InvalidParameter))?;
                pokemon_list.remove(index);
            }
            //Print to console the change for debugging purposes
            println!("Pokemon with id {} deleted", id); 
            load_list().await;

            Ok(warp::reply::json(&format!("Pokemon with id {} deleted", id)))
        },
        None => Err(warp::reject::custom(InvalidParameter)),
    }
}

async fn create_pokemon(id: i32, pokemon: Pokemon) -> Result<impl warp::Reply, warp::Rejection> {
    // Return the created Pokemon as a JSON response
    let new_pokemon = {
        let mut pokemon_list = POKEMON_LIST.lock().map_err(|_| warp::reject::custom(InvalidParameter))?;
        
        // Update the ID of the provided Pokemon object
        let new_pokemon = Pokemon {
            id,
            name: pokemon.name.clone(), 
            evolutions: pokemon.evolutions.clone(),
        };

        pokemon_list.push(new_pokemon.clone()); // Add the Pokemon to the list
        new_pokemon
    };

    // Add to the DB
    // we should check if the pokemon already exists in the DB
    let exists = get_if_pokemon_exists_in_mysql(new_pokemon.id).await.map_err(|_| warp::reject::custom(InvalidParameter))?;

    if !exists {
        create_pokemon_in_mysql(new_pokemon.clone()).await.map_err(|_| warp::reject::custom(InvalidParameter))?;
        // Print in console the new Pokemon (for debugging purposes)
        println!("New Pokemon: {:?}", new_pokemon);
        load_list().await;
    }
    else{
        println!("Pokemon with id {} already exists", new_pokemon.id);
        Err(warp::reject::custom(InvalidParameter))?;
    }

    Ok(warp::reply::json(&new_pokemon))
}


#[tokio::main]
async fn main() {
    // Load the Pokemon list from the database
    load_list().await;

    let get_path = warp::path("api");

    let get_version = warp::path("v1");

    let get_pokemon_by_id = warp::get()
        .and(warp::path!("pokemon" / i32))
        .and_then(get_pokemon_by_id);

    let put_pokemon = warp::put()
        .and(warp::path!("pokemon" / i32))
        .and(warp::body::json())
        .and_then(update_pokemon_by_id);

    let get_pokemon_list = warp::get()
        .and(warp::path("pokemon"))
        .and_then(get_pokemon_list);

    let post_pokemon = warp::post()
        .and(warp::path!("pokemon" / i32))
        .and(warp::body::json())
        .and_then(create_pokemon);

    let delete_pokemon = warp::delete()
        .and(warp::path!("pokemon" / i32))
        .and_then(delete_pokemon_by_id);

    let patch_pokemon = warp::patch()
        .and(warp::path!("pokemon" / i32))
        .and(warp::body::json())
        .and_then(update_pokemon_by_id);

    // Filter for the "/api" path that handles various HTTP methods
    let api_route = get_path.and(get_version).and(
            // GET request handler for specific ID
            get_pokemon_by_id
            // request handler for all Pokemon
            .or(get_pokemon_list)
            // request handlers under "/pokemon/{id}"
            .or(put_pokemon)
            .or(post_pokemon)
            .or(delete_pokemon)
            .or(patch_pokemon)
    );

    println!("Server is now running");

    //Start the Warp server, binding it to the address 127.0.0.1:3030
    warp::serve(api_route).run(([127, 0, 0, 1], 3030)).await;
    
}
