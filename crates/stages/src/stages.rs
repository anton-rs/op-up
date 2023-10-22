use eyre::Result;
use std::sync::Arc;

use op_config::Config;
use op_primitives::genesis;
use op_primitives::{Artifacts, Monorepo};

#[doc(hidden)]
pub mod allocs;
#[doc(hidden)]
pub mod contracts;
#[doc(hidden)]
pub mod deploy_config;
#[doc(hidden)]
pub mod directories;
#[doc(hidden)]
pub mod prestate;

#[doc(hidden)]
pub mod l1_exec;
#[doc(hidden)]
pub mod l1_genesis;

#[doc(hidden)]
pub mod l2_exec;
#[doc(hidden)]
pub mod l2_genesis;

#[doc(hidden)]
pub mod batcher;
#[doc(hidden)]
pub mod challenger;
#[doc(hidden)]
pub mod proposer;
#[doc(hidden)]
pub mod rollup;
#[doc(hidden)]
pub mod stateviz;

/// Stages
///
/// This module contains the code for the stages of the stack.
#[derive(Debug)]
pub struct Stages<'a> {
    /// The stack config.
    pub config: Config<'a>,
    /// The inner stages.
    pub inner: Option<Vec<Box<dyn crate::Stage>>>,
    /// A docker composer.
    pub composer: Option<Arc<op_composer::Composer>>,
}

impl Stages<'_> {
    /// Build the default docker-based stages.
    pub fn docker(
        &self,
        artifacts: Arc<Artifacts>,
        monorepo: Arc<Monorepo>,
        composer: Arc<op_composer::Composer>,
    ) -> Vec<Box<dyn crate::Stage>> {
        let genesis_timestamp = genesis::current_timestamp();

        vec![
            Box::new(directories::Directories::new(
                Arc::clone(&artifacts),
                Arc::clone(&monorepo),
            )),
            Box::new(prestate::Prestate::new(Arc::clone(&monorepo))),
            Box::new(allocs::Allocs::new(
                Arc::clone(&artifacts),
                Arc::clone(&monorepo),
            )),
            Box::new(deploy_config::DeployConfig::new(
                Arc::clone(&monorepo),
                genesis_timestamp,
            )),
            Box::new(l1_genesis::L1Genesis::new(
                Arc::clone(&monorepo),
                genesis_timestamp,
            )),
            Box::new(l1_exec::Executor::new(
                self.config.l1_client_port,
                self.config.l1_client,
                composer,
                Arc::clone(&artifacts),
            )),
            Box::new(l2_genesis::L2Genesis::new(
                self.config.l1_client_url.clone(),
                Arc::clone(&monorepo),
            )),
            Box::new(contracts::Contracts::new()),
            Box::new(l2_exec::Executor::new(
                self.config.l2_client_port,
                self.config.l2_client.to_string(),
            )),
            Box::new(rollup::Rollup::new(
                self.config.rollup_client_port,
                self.config.rollup_client.to_string(),
            )),
            Box::new(proposer::Proposer::new(Arc::clone(&artifacts))),
            Box::new(batcher::Batcher::new(
                Arc::clone(&artifacts),
                Arc::clone(&monorepo),
            )),
            Box::new(challenger::Challenger::new(
                Arc::clone(&artifacts),
                self.config.challenger.to_string(),
            )),
            Box::new(stateviz::Stateviz::new(Arc::clone(&artifacts))),
        ]
    }

    /// Execute the stages of the stack.
    pub async fn execute(&self) -> eyre::Result<()> {
        tracing::debug!(target: "stages", "executing stages");

        let monorepo = Arc::new(Monorepo::with_config(self.config.monorepo.clone())?);

        let composer = self
            .composer
            .clone()
            .unwrap_or(Arc::new(op_composer::Composer::new()?));

        // todo: fix this to use the stack config once the artifacts directory is configurable in
        // docker containers.
        let artifacts = Arc::new(Artifacts::from(
            std::env::current_dir()?.join(".devnet").as_path(),
        ));
        // let artifacts = Arc::new(Artifacts::from(self.config.artifacts.as_path()));

        let docker_stages = self.docker(artifacts, monorepo, composer);
        let inner = self.inner.as_ref().unwrap_or(&docker_stages);

        for stage in inner {
            stage.execute().await?;
        }

        tracing::info!(target: "stages", "finished executing stages");
        Ok(())
    }

    /// Print the stack result to stdout.
    pub fn output(&self) -> Result<()> {
        let l1_url = self.config.l1_client_url.clone();
        let l1_url = l1_url.unwrap_or(op_config::L1_URL.to_string());
        let l2_url = self.config.l2_client_url.clone();
        let l2_url = l2_url.unwrap_or(op_config::L2_URL.to_string());
        let rollup_url = self.config.rollup_client_url.clone();
        let rollup_url = rollup_url.unwrap_or(op_config::ROLLUP_URL.to_string());
        tracing::info!(target: "stages", "\n--------------------------");
        tracing::info!(target: "stages", "Devnet built successfully!");
        tracing::info!(target: "stages", "L1 endpoint: {}", l1_url);
        tracing::info!(target: "stages", "L2 endpoint: {}", l2_url);
        tracing::info!(target: "stages", "Rollup node endpoint: {}", rollup_url);
        tracing::info!(target: "stages", "--------------------------\n");
        Ok(())
    }
}

impl<'a> From<Config<'a>> for Stages<'a> {
    fn from(config: Config<'a>) -> Self {
        Self {
            config,
            inner: None,
            composer: None,
        }
    }
}
