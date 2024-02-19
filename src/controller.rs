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

pub static EPR_FINALIZER: &str = "endpointreferences.milkshakes.cloud";

#[derive(CustomResource, Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[cfg_attr(test, derive(Default))]
#[kube(kind = "EndpointReference", group = "milkshakes.cloud", version = "v1", namespaced)]
#[kube(status = "EndpointReferenceStatus", shortname = "endpointref")]
pub struct EndpointReferenceSpec {
    pub name: String,
    pub address: String,
    pub port: u16
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct EndpointReferenceStatus {
    pub active: bool
}

#[derive(Clone)]
pub struct Context {
    pub client: Client
}


fn error_policy(doc: Arc<EndpointReference>, error: &Error, ctx: Arc<Context>) -> Action {
    println!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

async fn reconcile(epr: Arc<EndpointReference>, ctx: Arc<Context>) -> Result<Action> {
    let ns = epr.namespace().unwrap();
    let docs: Api<EndpointReference> = Api::namespaced(ctx.client.clone(), &ns);

    println!("Reconciling Document \"{}\" in {}", epr.name_any(), ns);
    finalizer(&docs, EPR_FINALIZER, epr, |event| async {
        match event {
            Finalizer::Apply(doc) => doc.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
        .await
        .map_err(|e| Error::FinalizerError(Box::new(e)))
}

impl EndpointReference {
    async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action> {
        todo!()
    }

    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action> {
        todo!()
    }
}

#[derive(Clone, Default)]
pub struct State {}

impl State {
    pub fn to_context(&self, client: Client) -> Arc<Context> {
        Arc::new(Context {
            client
        })
    }
}

pub async fn run(state: State) {
    let client = Client::try_default().await.expect("failed to create kube Client");
    let docs = Api::<EndpointReference>::all(client.clone());
    if let Err(e) = docs.list(&ListParams::default().limit(1)).await {
        println!("CRD is not queryable; {e:?}. Is the CRD installed?");
        println!("Installation: cargo run --bin crdgen | kubectl apply -f -");
        std::process::exit(1);
    }
    Controller::new(docs, Config::default().any_semantic())
        .shutdown_on_signal()
        .run(reconcile, error_policy, state.to_context(client))
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;
}