use std::process::Child;
use std::sync::Arc;
use std::time::Duration;

use kubelet::container::ContainerKey;
use kubelet::pod::Pod;
use kubelet::state::{State, Transition};
use kubelet::state::prelude::*;
use log::{debug, error, info, warn};
use tokio::time::timeout;

use crate::provider::error::StackableError;
use crate::provider::PodState;
use crate::provider::states::failed::Failed;
use crate::provider::states::install_package::Installing;
use crate::provider::states::stopping::Stopping;

#[derive(Debug, TransitionTo)]
#[transition_to(Stopping, Failed, Running, Installing)]
pub struct Running {
    pub process_handle: Option<Child>,
}

#[async_trait::async_trait]
impl State<PodState> for Running {
    async fn next(mut self: Box<Self>, pod_state: &mut PodState, _pod: &Pod) -> Transition<PodState> {

        debug!("waiting");
        let mut handle = std::mem::replace(&mut self.process_handle, None).unwrap();
        /*while let Ok(_) = timeout(Duration::from_millis(100), changed.notified()).await {
            debug!("drained a waiting notification");
        }*/
       // debug!("done draining");

        loop {
            println!("running");
            tokio::select! {
                /*_ = changed.notified() => {
                    debug!("pod changed");
                    break;
                },*/
                _ = tokio::time::delay_for(std::time::Duration::from_secs(1))  => {
                    debug!("timer expired");
                }
            }
            match handle.try_wait() {
                Ok(None) => debug!("Still running"),
                _ => {
                    error!("died");
                    return Transition::next(self, Failed { message: "process died".to_string() })
                }

            }
        }
        Transition::next(self, Installing{
            download_directory: pod_state.download_directory.clone(),
            parcel_directory: pod_state.parcel_directory.clone(),
            package: pod_state.package.clone()
        })
   }

    async fn json_status(
        &self,
        _pod_state: &mut PodState,
        _pod: &Pod,
    ) -> anyhow::Result<serde_json::Value> {
        make_status(Phase::Running, &"status:running")
    }
}