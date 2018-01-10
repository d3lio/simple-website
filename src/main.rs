#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rand;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

mod game;
mod id_pool;

use std::sync::Mutex;
use std::collections::HashMap;

use rocket::State;
use rocket_contrib::{Json, Value};

use game::{Game, GameResult};
use id_pool::{Id, IdPool};
use rand::{thread_rng, Rng};

type GameMap = HashMap<Id, Game>;

#[derive(Deserialize)]
struct GameParams {
    length: u8,
    attempts: u32
}

#[derive(Deserialize)]
struct GuessParams {
    guess: String
}

const NUMBERS: [char; 9] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[post("/", format = "application/json", data="<params>")]
fn start(params: Json<GameParams>,
    games: State<Mutex<GameMap>>,
    id_pool: State<IdPool>) -> Result<Json<Value>, Json<Value>>
{
    let params = params.into_inner();

    if params.length < 4 || params.length > 9 {
        Err(Json(json!({
            "error": "Sequence length must be between 4 and 10."
        })))
    } else {
        let mut games = games.lock().expect("games lock failed");

        let id = id_pool.next();
        let mut numbers = NUMBERS.clone();
        thread_rng().shuffle(&mut numbers);
        let answer = &numbers[..params.length as usize];

        match Game::new(answer.to_vec(), params.attempts) {
            Ok(game) => {
                games.insert(id, game);

                Ok(Json(json!({
                    "details": format!("Game started with id {id}", id = id)
                })))
            },
            Err(game::Error::NonUnique) => {
                Err(Json(json!({
                    "error": "Sequence contains non unique characters."
                })))
            },
            _ => unreachable!()
        }
    }
}

#[post("/<id>", format = "application/json", data="<params>")]
fn guess(id: Id, params: Json<GuessParams>, games: State<Mutex<GameMap>>) -> Result<Json<Value>, Json<Value>> {
    let guess = params.into_inner().guess;

    let mut games = games.lock().expect("games lock failed");
    if let Some(game) = games.get_mut(&id) {
        match game.guess(guess.chars().collect::<Vec<_>>()) {
            Ok(GameResult::Feedback { bulls, cows }) => {
                Ok(Json(json!({
                    "details": {
                        "bulls": bulls,
                        "cows": cows
                    }
                })))
            },
            Ok(GameResult::Win) => {
                Ok(Json(json!({
                    "details": {
                        "message": "You Win! :)"
                    }
                })))
            },
            Ok(GameResult::Loss) => {
                Ok(Json(json!({
                    "details": {
                        "message": "You Lost :("
                    },
                    "answer": game.sequence().clone().into_iter().collect::<String>()
                })))
            },
            Err(game::Error::LenMismatch) => {
                Err(Json(json!({
                    "error": "Sequence is not the same length as the target."
                })))
            },
            Err(game::Error::NonUnique) => {
                Err(Json(json!({
                    "error": "Sequence contains non unique characters."
                })))
            },
        }
    } else {
        Err(Json(json!({
            "error": format!("Game with id {} does not exist", id)
        })))
    }
}

#[get("/<id>")]
fn history(id: Id, games: State<Mutex<GameMap>>) -> Result<Json<Value>, Json<Value>> {
    let games = games.lock().expect("games lock failed");
    if let Some(game) = games.get(&id) {
        Ok(Json(json!({
            "history": game.history()
        })))
    } else {
        Err(Json(json!({
            "error": format!("Game with id {} does not exist", id)
        })))
    }
}

#[get("/")]
fn index() -> &'static str {
    r"Commands:
    * GET / - display this page
    * POST / { length: u8 } - start a game
    * GET /<id> - retrieve history for a game
    * POST /<id> { guess: String } - guess for a started game"
}

#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "error": "Resource was not found."
    }))
}

fn main() {
    let games = Mutex::new(HashMap::<Id, Game>::new());
    let pool = IdPool::new();

    rocket::ignite()
        .manage(games)
        .manage(pool)
        .mount("/", routes![index, start, guess, history])
        .catch(errors![not_found])
        .launch();
}
