use crate::{Result, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use kube::{
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as Finalizer},
        watcher::Config,
    },
    CustomResource,
};
use std::sync::Arc;
use tokio::time::Duration;
use futures::StreamExt;
use serde_json::json;

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
pub struct EndpointContext {
    pub client: Client
}


pub fn eps_error_policy(_epr: Arc<EndpointReference>, error: &Error, _ctx: Arc<EndpointContext>) -> Action {
    println!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

pub async fn reconcile(epr: Arc<EndpointReference>, ctx: Arc<EndpointContext>) -> Result<Action> {
    let ns = epr.namespace().unwrap();
    let eprs: Api<EndpointReference> = Api::namespaced(ctx.client.clone(), &ns);

    println!("Reconciling EndpointReference \"{}\" in {}", epr.name_any(), ns);
    finalizer(&eprs, EPR_FINALIZER, epr, |event| async {
        match event {
            Finalizer::Apply(doc) => doc.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
        .await
        .map_err(|e| Error::FinalizerError(Box::new(e)))
}

impl EndpointReference {
    async fn reconcile(&self, ctx: Arc<EndpointContext>) -> Result<Action> {
        let client = ctx.client.clone();
        let ns = self.namespace().unwrap();
        let name = self.name_any();
        let erps: Api<EndpointReference> = Api::namespaced(client, &ns);

        if name == "illegal" {
            return Err(Error::InvalidEndpointReference);
        }

        let status = EndpointReferenceStatus {
            active: true
        };

        let new_status = Patch::Apply(json!({
            "apiVersion": "milkshakes.cloud/v1",
            "kind": "EndpointReference",
            "status": status
        }));
        let ps = PatchParams::apply("cntrlr").force();
        let _o = erps
            .patch_status(&name, &ps, &new_status)
            .await
            .map_err(Error::KubeError)?;

        Ok(Action::requeue(Duration::from_secs(60)))
    }

    async fn cleanup(&self, _ctx: Arc<EndpointContext>) -> Result<Action> {
        // EndpointReference doesn't have any real cleanup, so we just publish an event
        Ok(Action::await_change())
    }
}

#[derive(Clone, Default)]
pub struct EndpointState {}

impl EndpointState {
    pub fn to_context(&self, client: Client) -> Arc<EndpointContext> {
        Arc::new(EndpointContext {
            client
        })
    }
}

pub async fn run_endpoints_controller(state: EndpointState) {
    let client = Client::try_default().await.expect("failed to create kube Client");
    let eprs = Api::<EndpointReference>::all(client.clone());
    if let Err(e) = eprs.list(&ListParams::default().limit(1)).await {
        println!("CRD is not queryable; {e:?}. Is the CRD installed?");
        println!("Installation: cargo run --bin crdgen | kubectl apply -f -");
        std::process::exit(1);
    }
    Controller::new(eprs, Config::default().any_semantic())
        .shutdown_on_signal()
        .run(reconcile, eps_error_policy, state.to_context(client))
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;
}