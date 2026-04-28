use crate::clients::email::EmailClient;
use crate::command::permissions::sync::sync_permissions;
use crate::routes::api::confirmations::login::delete::delete_confirmations_login;
use crate::routes::api::confirmations::login::post::post_confirmations_login;
use crate::routes::api::confirmations::registration::delete::delete_confirmations_registration;
use crate::routes::api::confirmations::registration::post::post_confirmations_registration;
use crate::routes::api::confirmations::user_update::password::delete::delete_confirmations_user_update_password;
use crate::routes::api::confirmations::user_update::password::post::post_confirmations_user_update_password;
use crate::routes::api::session::user::get::get_session;
use crate::routes::api::session::user::password::put::put_session_user_password;
use crate::settings::Settings;
use crate::routes::api::session::post::post_session;
use crate::routes::api::users::post::post_users;
use crate::routes::api::health_check::check_health;
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
        let application_settings = configuration.application_settings();
        let address = format!(
            "{}:{}",
            application_settings.host, application_settings.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        // init connections
        let db_pool = Self::database_connection(configuration.clone());
        let email_client = Self::email_client(configuration.clone());

        sync_permissions(&db_pool, &configuration.permissions).await?;

        let server = Self::get_server(
            listener,
            db_pool,
            email_client,
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
        db_pool: PgPool,
        email_client: EmailClient,
        config: Settings
    ) -> Result<Server, anyhow::Error> {
        let config = Data::new(config);
        let db_pool = Data::new(db_pool);
        let email_client = Data::new(email_client);
        
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .app_data(config.clone())
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
                .service(web::scope("/api")
                    .route("/health", web::get().to(check_health))
                    .route("/users", web::post().to(post_users))
                    .service(web::scope("/session")
                        .route("", web::post().to(post_session))
                        .service(web::scope("/user")
                            .route("", web::get().to(get_session))
                            .route("/password", web::put().to(put_session_user_password))
                        )
                    )
                    .service(web::scope("/confirmations")
                        .service(web::scope("/registration")
                            .route("/{confirmation_id}", web::post().to(post_confirmations_registration))
                            .route("/{confirmation_id}", web::delete().to(delete_confirmations_registration))
                        )
                        .service(web::scope("/login")
                            .route("/{confirmation_id}", web::post().to(post_confirmations_login))
                            .route("/{confirmation_id}", web::delete().to(delete_confirmations_login))
                        )
                        .service(web::scope("/user_update")
                            .service(web::scope("/password")
                                .route("/{confirmation_id}", web::post().to(post_confirmations_user_update_password))
                                .route("/{confirmation_id}", web::delete().to(delete_confirmations_user_update_password))
                            )
                        )
                    )
                )
        })
        .listen(listener)?
        .run();
        Ok(server)
    }
}