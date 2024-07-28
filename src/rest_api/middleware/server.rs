use std::net::ToSocketAddrs;
use std::sync::Arc;

use crate::http::client::HttpClient;
use crate::http::server::HttpServer;

use crate::drivers::server::prelude::*;

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, Hash)]
/// Server HTTP middleware
/// 
/// This struct is used to process HTTP REST API requests
/// to the inner server driver.
pub struct Server<HttpClientExt, HttpServerExt, RouterExt, TraversalExt, MessagesInboxExt> {
    http_client: HttpClientExt,
    http_server: HttpServerExt,
    driver: Arc<ServerDriver<RouterExt, TraversalExt, MessagesInboxExt>>
}

impl<HttpClientExt, HttpServerExt, RouterExt, TraversalExt, MessagesInboxExt>
    Server<HttpClientExt, HttpServerExt, RouterExt, TraversalExt, MessagesInboxExt>
where
    HttpClientExt: HttpClient,
    HttpServerExt: HttpServer,
    RouterExt: Router + Send + Sync + 'static,
    TraversalExt: Traversal + Send + Sync + 'static,
    MessagesInboxExt: MessagesInbox + Send + Sync + 'static,
{
    pub async fn new(
        http_client: HttpClientExt,
        mut http_server: HttpServerExt,
        server_driver: ServerDriver<RouterExt, TraversalExt, MessagesInboxExt>
    ) -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            http_client_type = std::any::type_name::<HttpClientExt>(),
            http_server_type = std::any::type_name::<HttpServerExt>(),
            router_type = std::any::type_name::<RouterExt>(),
            traversal_type = std::any::type_name::<TraversalExt>(),
            messages_inbox_type = std::any::type_name::<MessagesInboxExt>(),
            server_address = server_driver.params().address,
            server_secret = server_driver.params().secret_key.to_base64(),
            "Building server REST API middleware"
        );

        let driver = Arc::new(server_driver);

        http_server.get("/api/v1/info", {
            let driver = driver.clone();

            |client_address| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "GET /api/v1/info");

                InfoResponse::new(&driver.params().secret_key)
            }
        }).await;

        http_server.get("/api/v1/clients", {
            let driver = driver.clone();

            |client_address| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "GET /api/v1/clients");

                let clients = driver.router()
                    .local_clients().await
                    .unwrap_or_default();

                #[cfg(feature = "tracing")]
                tracing::trace!("GET /api/v1/clients: returned {} records", clients.len());

                ClientsResponse::new(clients)
            }
        }).await;

        http_server.get("/api/v1/servers", {
            let driver = driver.clone();

            |client_address| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "GET /api/v1/servers");

                let servers = driver.router()
                    .servers().await
                    .unwrap_or_default();

                #[cfg(feature = "tracing")]
                tracing::trace!("GET /api/v1/servers: returned {} records", servers.len());

                ServersResponse::new(servers)
            }
        }).await;

        http_server.post::<ConnectRequest, ConnectResponse, _>("/api/v1/connect", {
            let driver = driver.clone();

            |client_address, request: ConnectRequest| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/connect");

                // Validate incoming request
                let validated = match request.validate(&driver.params().secret_key.public_key()) {
                    Ok(validated) => validated,

                    Err(err) => return ConnectResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to validate request: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return ConnectResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Index client in the routing table
                let client = Client::new(
                    request.0.public_key,
                    request.0.request.certificate,
                    request.0.request.client
                );

                #[cfg(feature = "tracing")]
                tracing::trace!(
                    client_public = client.public_key.to_base64(),
                    client_info = std::any::type_name_of_val(&client.info),
                    "POST /api/v1/connect: indexing local client"
                );

                if let Err(err) = driver.router().index_local_client(client).await {
                    return ConnectResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to index local client: {err}")
                    );
                }

                ConnectResponse::success(
                    ResponseStatus::Success,
                    &driver.params().secret_key,
                    request.0.proof_seed
                )
            }
        }).await;

        http_server.post::<DisconnectRequest, DisconnectResponse, _>("/api/v1/disconnect", {
            let driver = driver.clone();

            |client_address, request: DisconnectRequest| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/disconnect");

                // Validate incoming request
                let validated = match request.validate() {
                    Ok(validated) => validated,

                    Err(err) => return DisconnectResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to validate request: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return DisconnectResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                #[cfg(feature = "tracing")]
                tracing::trace!(
                    client_public = request.0.public_key.to_base64(),
                    "POST /api/v1/disconnect: disconnecting client"
                );

                if let Err(err) = driver.router().disconnect(&request.0.public_key).await {
                    return DisconnectResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to disconnect client: {err}")
                    );
                }

                DisconnectResponse::success(
                    ResponseStatus::Success,
                    &driver.params().secret_key,
                    request.0.proof_seed
                )
            }
        }).await;

        http_server.post::<AnnounceRequest, AnnounceResponse, _>("/api/v1/announce", {
            let driver = driver.clone();

            |client_address, request: AnnounceRequest| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/announce");

                // Validate incoming request
                let validated = match request.validate() {
                    Ok(validated) => validated,

                    Err(err) => return AnnounceResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to validate request: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return AnnounceResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Index client in the routing table
                match request.0.request {
                    AnnounceRequestBody::Client { client, server } => {
                        if let Err(err) = driver.router().index_remote_client(client, server).await {
                            return AnnounceResponse::error(
                                ResponseStatus::ServerError,
                                format!("Failed to index remote client: {err}")
                            );
                        }
                    }

                    AnnounceRequestBody::Server { server } => {
                        if let Err(err) = driver.router().index_server(server).await {
                            return AnnounceResponse::error(
                                ResponseStatus::ServerError,
                                format!("Failed to index server: {err}")
                            );
                        }
                    }
                }

                AnnounceResponse::success(
                    ResponseStatus::Success,
                    &driver.params().secret_key,
                    request.0.proof_seed
                )
            }
        }).await;

        http_server.post::<LookupRequest, LookupResponse, _>("/api/v1/lookup", {
            let driver = driver.clone();

            |client_address, request: LookupRequest| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/lookup");

                // Validate incoming request
                let validated = match request.validate() {
                    Ok(validated) => validated,

                    Err(err) => return LookupResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to validate request: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return LookupResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Try to find the client in the local index
                match driver.router().lookup_local_client(&request.0.public_key, request.0.request.client_type).await {
                    Ok(Some((client, available))) => {
                        let body = LookupResponseBody::local(client, available);

                        return LookupResponse::success(
                            ResponseStatus::Success,
                            &driver.params().secret_key,
                            request.0.proof_seed,
                            body
                        );
                    }

                    Err(err) => return LookupResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to lookup local client: {err}")
                    ),

                    _ => ()
                }

                // Try to find the client in the remote index
                match driver.router().lookup_remote_client(&request.0.public_key, request.0.request.client_type).await {
                    Ok(Some((client, server, available))) => {
                        let body = LookupResponseBody::remote(client, server, available);

                        return LookupResponse::success(
                            ResponseStatus::Success,
                            &driver.params().secret_key,
                            request.0.proof_seed,
                            body
                        );
                    }

                    Err(err) => return LookupResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to lookup remote client: {err}")
                    ),

                    _ => ()
                }

                // Return searching hint if neither local nor known remote record found
                let hint = driver.router()
                    .lookup_remote_client_hint(&request.0.public_key, request.0.request.client_type)
                    .await;

                match hint {
                    Ok(hint) => LookupResponse::success(
                        ResponseStatus::Success,
                        &driver.params().secret_key,
                        request.0.proof_seed,
                        LookupResponseBody::hint(hint)
                    ),

                    Err(err) => LookupResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to lookup remote client hint: {err}")
                    )
                }
            }
        }).await;

        http_server.post::<SendRequest, SendResponse, _>("/api/v1/send", {
            let driver = driver.clone();

            |client_address, request: SendRequest| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/send");

                // Validate incoming request
                let validated = match request.validate() {
                    Ok(validated) => validated,

                    Err(err) => return SendResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to validate request: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return SendResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Add message to the inbox
                let result = driver.messages_inbox().add_message(
                    request.0.request.sender,
                    request.0.request.receiver_public,
                    request.0.request.channel,
                    request.0.request.message
                ).await;

                match result {
                    Ok(()) => SendResponse::success(
                        ResponseStatus::Success,
                        &driver.params().secret_key,
                        request.0.proof_seed
                    ),

                    Err(err) => SendResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to index message: {err}")
                    )
                }
            }
        }).await;

        http_server.post::<PollRequest, PollResponse, _>("/api/v1/poll", {
            let driver = driver.clone();

            |client_address, request: PollRequest| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/poll");

                // Validate incoming request
                let validated = match request.validate() {
                    Ok(validated) => validated,

                    Err(err) => return PollResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to validate request: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return PollResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Poll messages from the inbox
                let messages = driver.messages_inbox().poll_messages(
                    request.0.public_key,
                    request.0.request.channel,
                    request.0.request.limit
                ).await;

                match messages {
                    Ok((messages, remaining)) => PollResponse::success(
                        ResponseStatus::Success,
                        &driver.params().secret_key,
                        request.0.proof_seed,
                        PollResponseBody::new(messages, remaining)
                    ),

                    Err(err) => PollResponse::error(
                        ResponseStatus::ServerError,
                        format!("Failed to poll messages: {err}")
                    )
                }
            }
        }).await;

        Self {
            http_client,
            http_server,
            driver
        }
    }

    #[inline]
    pub fn http_client(&self) -> &HttpClientExt {
        &self.http_client
    }

    #[inline]
    pub fn http_server(&self) -> &HttpServerExt {
        &self.http_server
    }

    #[inline]
    pub fn driver(&self) -> Arc<ServerDriver<RouterExt, TraversalExt, MessagesInboxExt>> {
        self.driver.clone()
    }

    #[inline]
    /// Run HTTP REST API server on given TCP listener
    pub async fn serve(self, address: impl ToSocketAddrs + Send) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Starting server");

        self.http_server.serve(address).await
    }
}
