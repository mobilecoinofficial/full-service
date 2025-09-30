use crate::query::QueryManager;
use grpcio::{RpcContext, RpcStatus, Service, UnarySink};
use mc_common::logger::{log, Logger};
use mc_full_service_mirror::wallet_service_mirror_api::{
    create_wallet_service_mirror, PollRequest, PollResponse, WalletServiceMirror,
};
use mc_util_grpc::{rpc_logger, send_result};

#[derive(Clone)]
pub struct MirrorService {
    /// Query manager.
    query_manager: QueryManager,

    /// Logger.
    logger: Logger,
}

impl MirrorService {
    pub fn new(query_manager: QueryManager, logger: Logger) -> Self {
        Self {
            query_manager,
            logger,
        }
    }

    pub fn into_service(self) -> Service {
        create_wallet_service_mirror(self)
    }

    fn poll_impl(&self, request: PollRequest, logger: &Logger) -> Result<PollResponse, RpcStatus> {
        // Go over any responses we may have received and attempt to resolve them.
        for (query_id, query_response) in request.query_responses.iter() {
            match self.query_manager.resolve_query(query_id, query_response) {
                Ok(()) => log::info!(logger, "Query {} resolved", query_id),
                Err(err) => log::error!(logger, "Query {} failed resolving: {}", query_id, err),
            }
        }

        // Return any queries we have received.
        let pending_requests = self.query_manager.get_pending_requests();

        log::debug!(
            logger,
            "Polled with {} returned responses and {} new requests",
            request.query_responses.len(),
            pending_requests.len()
        );

        Ok(PollResponse {
            query_requests: pending_requests.into_iter().collect(),
        })
    }
}

impl WalletServiceMirror for MirrorService {
    fn poll(&mut self, ctx: RpcContext, request: PollRequest, sink: UnarySink<PollResponse>) {
        let logger = rpc_logger(&ctx, &self.logger);
        send_result(ctx, sink, self.poll_impl(request, &logger), &logger)
    }
}
