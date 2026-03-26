use crate::clients::email::EmailClient;
use crate::routes::api::session::get::get_session;
use crate::routes::api::session::user::password::put::put_session_user_password;
use crate::settings::Settings;
use crate::routes::api::login_confirmations::post::post_confirmations_login;
use crate::routes::api::session::post::post_session;
use crate::routes::api::registration_confirmations::delete::delete_confirmations_registration;
use crate::routes::api::registration_confirmations::post::post_confirmations_registration;
use crate::routes::api::users::post::post_users;
use crate::routes::api::health_check::health_check;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct ApplicationBaseUrl(pub String);

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    /// Creates a connection to Postgres database and binds TCP listener using given configuration.
    pub async fn configure(configuration: Settings) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = Self::get_server(
            listener,
            configuration
        )
        .await?;

        Ok(Self { port, server })
    }

    /// Run the server indefinetly
    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    /// get the server port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get a database connection
    pub fn database_connection(configuration: Settings) -> PgPool {
        PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
    }

    pub fn email_client(configuration: Settings) -> EmailClient {
        EmailClient::new(configuration.email.server).unwrap()
    }

    /// Get the server (actix_web::HttpServer) instance
    async fn get_server(
        listener: TcpListener,
        configuration: Settings
    ) -> Result<Server, anyhow::Error> {
        let config = Data::new(configuration.clone());
        let db_pool = Data::new(Self::database_connection(configuration.clone()));
        let email_client = Data::new(Self::email_client(configuration.clone()));
        
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .app_data(config.clone())
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
                .service(web::scope("/api")
                    .route("/health", web::get().to(health_check))
                    .route("/users", web::post().to(post_users))
                    .service(web::scope("/confirmations")
                        .service(web::scope("/registration")
                            .route("/{confirmation_id}", web::post().to(post_confirmations_registration))
                            .route("/{confirmation_id}", web::delete().to(delete_confirmations_registration))
                        )
                        .service(web::scope("/login")
                            .route("/{confirmation_id}", web::post().to(post_confirmations_login))
                        )
                    )
                    .route("/session", web::post().to(post_session))
                    .route("/session", web::get().to(get_session))
                    .route("/session/user/password", web::put().to(put_session_user_password))
                )
        })
        .listen(listener)?
        .run();
        Ok(server)
    }
}