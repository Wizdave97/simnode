use sc_cli::{CliConfiguration};
use std::error::Error;
use std::sync::Arc;
use parachain_inherent::ParachainInherentData;
use parachain_runtime::BlockNumber;
use sc_consensus_manual_seal::consensus::timestamp::SlotTimestampProvider;
use sc_service::TFullBackend;
use substrate_simnode::{FullClientFor, RpcHandlerArgs, SignatureVerificationOverride};


/// Concrete header
pub type Header = sp_runtime::generic::Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;

/// Opaque block
pub type OpaqueBlock = sp_runtime::generic::Block<Header, sp_runtime::OpaqueExtrinsic>;
/// [`SimnodeCli`] implementation
pub struct TemplateCli;

impl substrate_simnode::SimnodeCli for TemplateCli {
    type CliConfig = sc_cli::RunCmd;
    type SubstrateCli = node::cli::Cli;

    fn cli_config(cli: &Self::SubstrateCli) -> &Self::CliConfig {
        &cli.run.base
    }

    fn log_filters(cli_config: &Self::CliConfig) -> Result<String, sc_cli::Error> {
        cli_config.log_filters()
    }
}

/// A unit struct which implements `NativeExecutionDispatch` feeding in the
/// hard-coded runtime.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
    type ExtendHostFunctions =
    (frame_benchmarking::benchmarking::HostFunctions, SignatureVerificationOverride);

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        parachain_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        parachain_runtime::native_version()
    }
}

/// ChainInfo implementation.
pub struct ChainInfo;

impl substrate_simnode::ChainInfo for ChainInfo {
    type Block = OpaqueBlock;
    type ExecutorDispatch = ExecutorDispatch;
    type Runtime = parachain_runtime::Runtime;
    type RuntimeApi = parachain_runtime::RuntimeApi;
    type SelectChain = sc_consensus::LongestChain<TFullBackend<Self::Block>, Self::Block>;
    type BlockImport = Arc<FullClientFor<Self>>;
    type InherentDataProviders = (
        SlotTimestampProvider,
        sp_consensus_aura::inherents::InherentDataProvider,
        ParachainInherentData,
    );
    type Cli = TemplateCli;

    fn create_rpc_io_handler<SC>(
        deps: RpcHandlerArgs<Self, SC>,
    ) -> jsonrpc_core::MetaIoHandler<sc_rpc::Metadata>
        where
            <<Self as substrate_simnode::ChainInfo>::RuntimeApi as sp_api::ConstructRuntimeApi<
                Self::Block,
                FullClientFor<Self>,
            >>::RuntimeApi: sp_api::Core<Self::Block>
            + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Self::Block>,
    {
        let full_deps = node::rpc::FullDeps {
            client: deps.client,
            pool: deps.pool,
            deny_unsafe: deps.deny_unsafe,
        };
        node::rpc::create_full::<_, _>(full_deps)
    }
}



fn main() -> Result<(), Box<dyn Error>> {
    substrate_simnode::parachain_node::<ChainInfo, _, _>(
        |node| async move {
            node.seal_blocks(10).await;
            node.until_shutdown().await;
            Ok(())
        },
    )?;

	Ok(())
}
