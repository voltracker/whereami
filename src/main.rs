use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::{de::Error, Deserialize, Serialize};
use serde_json::json;
use tokio_postgres::{GenericClient, NoTls};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost port=5432 user=postgres dbname=whereami password=mysecretpassword",
        NoTls,
    )
    .unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/", get(get_all_users))
        .route("/user", get(get_user).post(add_user))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn add_user(
    State(pool): State<ConnectionPool>,
    Json(input): Json<NewUser>,
) -> impl IntoResponse {
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to connect to database"})),
            )
        }
    };

    let NewUser {
        forename,
        surname,
        email,
        location,
    } = input;

    let Location { lat, long } = location;

    let res = conn
        .execute(
            "INSERT INTO users (first_name, last_name, email, latitude, longitude) VALUES ($1, $2, $3, $4, $5);",
            &[&forename, &surname, &email, &lat, &long],
        )
        .await;

    match res {
        Ok(info) => {
            tracing::info!("user added: {}", email);
            tracing::info!("DB: updated {} rows", info);
            (StatusCode::OK, Json(json!({"added:": email})))
        }
        Err(err) => {
            tracing::error!("DB: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to insert user"})),
            )
        }
    }
}

async fn get_all_users(State(pool): State<ConnectionPool>) -> impl IntoResponse {
    return match fetch_users(pool).await {
        Ok(users) => (StatusCode::OK, Json(json!(users))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error" : "failed to connect to database!"})),
        ),
    };
}

async fn get_user(State(pool): State<ConnectionPool>, Json(user): Json<User>) -> impl IntoResponse {
    let users = match fetch_users(pool).await {
        Ok(users) => users,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error" : "failed to connect to database!"})),
            )
        }
    };

    let locations = get_locations_for_user(users, user);

    (StatusCode::OK, Json(json!(locations)))
}

async fn fetch_users(pool: ConnectionPool) -> Result<Vec<User>, ()> {
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("DB error: {}", err);
            return Err(());
        }
    };

    let rows = match conn
        .query(
            "SELECT id, latitude, longitude, first_name, last_name, email FROM users",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!("DB: {}", err);
            return Err(());
        }
    };

    let mut users: Vec<User> = Vec::new();

    for row in rows.into_iter() {
        users.push(User {
            id: row.get(0),
            location: Location {
                lat: row.get(1),
                long: row.get(2),
            },
            forename: row.get(3),
            surname: row.get(4),
            email: row.get(5),
        })
    }
    return Ok(users);
}

fn get_mock_user() -> User {
    let usr = User {
        id: 1,
        location: Location {
            lat: 1.1,
            long: 1.1,
        },
        forename: String::from("matthew"),
        surname: String::from("davidson"),
        email: String::from("hello.world@example.com"),
    };
    return usr;
}

#[derive(Serialize, Deserialize)]
struct NewUser {
    location: Location,
    forename: String,
    surname: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    location: Location,
    forename: String,
    surname: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct Location {
    lat: f32,
    long: f32,
}

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

fn get_all_locations(users: Vec<User>) -> Vec<Location> {
    return users.into_iter().map(|user| user.location).collect();
}

fn get_locations_for_user(users: Vec<User>, requester: User) -> Vec<Location> {
    return users
        .into_iter()
        .filter(|u| requester.surname == u.surname && requester.forename == u.forename)
        .map(|u| u.location)
        .collect();
}
