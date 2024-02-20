use crate::{Result, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use kube::{
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        events::{Event, EventType, Recorder, Reporter},
        finalizer::{finalizer, Event as Finalizer},
        watcher::Config,
    },
    CustomResource, Resource,
};
use std::sync::Arc;
use tokio::time::Duration;
use futures::StreamExt;

pub static FRPC_FINALIZER: &str = "frpclients.milkshakes.cloud";

#[derive(CustomResource, Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[cfg_attr(test, derive(Default))]
#[kube(kind = "FrpClient", group = "milkshakes.cloud", version = "v1", namespaced)]
#[kube(status = "FrpClientStatus", shortname = "frpclient")]
pub struct FrpClientSpec {
    pub name: String,
    pub svr_address: String,
    pub svr_port: u16,
    pub tgt_port: u16,
    pub rem_port: u16
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct FrpClientStatus {
    pub active: bool,
    pub provisioned: bool
}

#[derive(Clone)]
pub struct FrpClientContext {
    pub client: Client
}

async fn reconcile_frpclients(frpc: Arc<FrpClient>, ctx: Arc<FrpClientContext>) -> Result<Action> {
    let ns = frpc.namespace().unwrap();
    let frpcs: Api<FrpClient> = Api::namespaced(ctx.client.clone(), &ns);

    println!("Reconciling FrpClient \"{}\" in {}", frpc.name_any(), ns);
    finalizer(&frpcs, FRPC_FINALIZER, frpc, |event| async {
        match event {
            Finalizer::Apply(doc) => doc.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
        .await
        .map_err(|e| Error::FinalizerError(Box::new(e)))
}

impl FrpClient {
    async fn reconcile(&self, _ctx: Arc<FrpClientContext>) -> Result<Action> {
        todo!()
    }

    async fn cleanup(&self, _ctx: Arc<FrpClientContext>) -> Result<Action> {
        todo!()
    }
}

#[derive(Clone, Default)]
pub struct FrpClientState {}

impl FrpClientState {
    pub fn to_context(&self, client: Client) -> Arc<FrpClientContext> {
        Arc::new(FrpClientContext {
            client
        })
    }
}

pub fn error_policy(_frpc: Arc<FrpClient>, error: &Error, _ctx: Arc<FrpClientContext>) -> Action {
    println!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

pub async fn run_frpclient_controller(state: FrpClientState) {
    let client = Client::try_default().await.expect("failed to create kube Client");
    let frpcs = Api::<FrpClient>::all(client.clone());
    if let Err(e) = frpcs.list(&ListParams::default().limit(1)).await {
        println!("CRD is not queryable; {e:?}. Is the CRD installed?");
        println!("Installation: cargo run --bin crdgen | kubectl apply -f -");
        std::process::exit(1);
    }
    Controller::new(frpcs, Config::default().any_semantic())
        .shutdown_on_signal()
        .run(reconcile_frpclients, error_policy, state.to_context(client))
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;
}