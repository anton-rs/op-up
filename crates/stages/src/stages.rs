use eyre::Result;
use std::rc::Rc;

use op_config::Config;
use op_primitives::genesis;
use op_primitives::{Artifacts, Monorepo};

#[doc(hidden)]
pub mod allocs;
#[doc(hidden)]
pub mod artifacts;
#[doc(hidden)]
pub mod cannon;
#[doc(hidden)]
pub mod contracts;
#[doc(hidden)]
pub mod deploy_config;
#[doc(hidden)]
pub mod directories;

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
}

impl Stages<'_> {
    /// Build the default docker-based stages.
    pub fn docker(
        &self,
        artifacts: Rc<Artifacts>,
        monorepo: Rc<Monorepo>,
    ) -> Vec<Box<dyn crate::Stage>> {
        let genesis_timestamp = genesis::current_timestamp();
        let l1_client = self.config.l1_client.to_string();
        let l2_client = self.config.l2_client.to_string();
        let rollup_client = self.config.rollup_client.to_string();
        let challenge_agent = self.config.challenger.to_string();
        vec![
            Box::new(artifacts::Artifacts::new(self.config.artifacts.clone())),
            Box::new(directories::Directories::new(Rc::clone(&monorepo))),
            Box::new(cannon::Prestate::new(Rc::clone(&monorepo))),
            Box::new(allocs::Allocs::new(
                Rc::clone(&artifacts),
                Rc::clone(&monorepo),
            )),
            Box::new(deploy_config::DeployConfig::new(
                Rc::clone(&monorepo),
                genesis_timestamp,
            )),
            Box::new(l1_genesis::L1Genesis::new(
                Rc::clone(&monorepo),
                genesis_timestamp,
            )),
            Box::new(l1_exec::Executor::new(l1_client)),
            Box::new(l2_genesis::L2Genesis::new(Rc::clone(&monorepo))),
            Box::new(contracts::Contracts::new()),
            Box::new(l2_exec::Executor::new(l2_client)),
            Box::new(rollup::Rollup::new(rollup_client)),
            Box::new(proposer::Proposer::new()),
            Box::new(batcher::Batcher::new(Rc::clone(&monorepo))),
            Box::new(challenger::Challenger::new(challenge_agent)),
            Box::new(stateviz::Stateviz::new()),
        ]
    }

    /// Execute the stages of the stack.
    pub async fn execute(&self) -> eyre::Result<()> {
        tracing::debug!(target: "stages", "executing stages");

        let monorepo = Rc::new(Monorepo::new()?);

        // todo: fix this to use the stack config once the artifacts directory is configurable in
        // docker containers.
        let artifacts = Rc::new(Artifacts::from(
            std::env::current_dir()?.join(".devnet").as_path(),
        ));
        // let artifacts = Rc::new(Artifacts::from(self.config.artifacts.as_path()));

        let docker_stages = self.docker(artifacts, monorepo);
        let inner = self.inner.as_ref().unwrap_or(&docker_stages);

        for stage in inner {
            stage.execute()?;
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
        }
    }
}
