use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::health_check::health_check;
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
        let connection_pool = Self::database_connection(&configuration.database);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = Self::get_server(
            listener,
            connection_pool,
            configuration.application.base_url,
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
    pub fn database_connection(configuration: &DatabaseSettings) -> PgPool {
        PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
    }

    /// Get the server (actix_web::HttpServer) instance
    async fn get_server(
        listener: TcpListener,
        db_pool: PgPool,
        base_url: String
    ) -> Result<Server, anyhow::Error> {
        let db_pool = Data::new(db_pool);
        let base_url = Data::new(ApplicationBaseUrl(base_url));

        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .app_data(db_pool.clone())
                .app_data(base_url.clone())
                .service(web::scope("/api")
                    .route("/health", web::get().to(health_check))
                )
        })
        .listen(listener)?
        .run();
        Ok(server)
    }
}