#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::{collections::HashMap, fmt::Debug};

use bollard::{
    container::{
        CreateContainerOptions, ListContainersOptions, LogOutput, RemoveContainerOptions,
        StartContainerOptions, StopContainerOptions,
    },
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    service::{ContainerConfig, ContainerCreateResponse, ContainerSummary},
    Docker,
};
use eyre::{bail, Result};
use futures_util::{StreamExt, TryStreamExt};
use serde::Serialize;

/// The Composer is responsible for managing the OP-UP docker containers.
#[derive(Debug)]
pub struct Composer {
    /// The Docker daemon client.
    pub daemon: Docker,
}

impl Composer {
    /// Create a new instance of the Composer.
    pub fn new() -> Result<Self> {
        let daemon = Docker::connect_with_local_defaults()?;

        tracing::debug!("Successfully connected to Docker daemon");
        Ok(Self { daemon })
    }

    /// List all the OP-UP docker containers existing on the host.
    ///
    /// The containers are filtered by the label `com.docker.compose.project=op-up`.
    ///
    /// This method allows optional filtering by container status:
    /// `created`|`restarting`|`running`|`removing`|`paused`|`exited`|`dead`
    pub async fn list_containers(&self, status: Option<&str>) -> Result<Vec<ContainerSummary>> {
        let mut filters = HashMap::new();
        filters.insert("label", vec!["com.docker.compose.project=op-up"]);

        if let Some(status) = status {
            filters.insert("status", vec![status]);
        }

        let list_options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        self.daemon
            .list_containers(Some(list_options))
            .await
            .map_err(Into::into)
    }

    /// Create a Docker image from the specified options.
    ///
    /// Returns the ID of the created image.
    pub async fn create_image<T>(&self, opts: CreateImageOptions<'_, T>) -> Result<String>
    where
        T: Into<String> + Serialize + Clone + Debug,
    {
        let res = self
            .daemon
            .create_image(Some(opts), None, None)
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                println!("Error creating docker image: {:?}", e);
                e
            })?;

        tracing::debug!("Created docker image: {:?}", res);

        match res.get(0) {
            Some(info) => match info.id.as_ref() {
                Some(id) => Ok(id.clone()),
                None => bail!("No image ID found in response"),
            },
            None => bail!("No image info found in response"),
        }
    }

    /// Create a Docker container for the specified OP Stack component
    pub async fn create_container(
        &self,
        name: &str,
        mut config: ContainerConfig,
    ) -> Result<ContainerCreateResponse> {
        let create_options = CreateContainerOptions {
            name,
            platform: None,
        };

        let labels = config.labels.get_or_insert_with(HashMap::new);
        labels.insert(
            "com.docker.compose.project".to_string(),
            "op-up".to_string(),
        );

        let res = self
            .daemon
            .create_container(Some(create_options), config.into())
            .await?;

        tracing::debug!("Created docker container {} with ID: {}", name, res.id);

        Ok(res)
    }

    /// Start the specified OP Stack component container by ID.
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.daemon
            .start_container(id, None::<StartContainerOptions<&str>>)
            .await?;

        tracing::debug!("Started docker container with ID: {}", id);
        Ok(())
    }

    /// Stop the specified OP Stack component container by ID.
    pub async fn stop_container(&self, id: &str) -> Result<()> {
        self.daemon
            .stop_container(id, None::<StopContainerOptions>)
            .await?;

        tracing::debug!("Stopped docker container with ID: {}", id);
        Ok(())
    }

    /// Remove the specified OP Stack component container by ID.
    pub async fn remove_container(&self, id: &str) -> Result<()> {
        self.daemon
            .remove_container(id, None::<RemoveContainerOptions>)
            .await?;

        tracing::debug!("Removed docker container with ID: {}", id);
        Ok(())
    }

    /// Stop all OP-UP docker containers at once.
    pub async fn stop_all_containers(&self) -> Result<()> {
        let running_containers = self.list_containers(Some("running")).await?;

        let ids = running_containers
            .iter()
            .filter_map(|container| container.id.as_ref())
            .map(|id| id.as_str())
            .collect::<Vec<_>>();

        tracing::info!("Stopping docker containers: {:?}", ids);

        for id in ids {
            self.daemon
                .stop_container(id, None::<StopContainerOptions>)
                .await?;

            tracing::debug!("Successfully stopped docker container: {}", id);
        }

        Ok(())
    }

    /// Remove all OP-UP docker containers at once
    pub async fn purge_all_containers(&self) -> Result<()> {
        let containers = self.list_containers(None).await?;

        let ids = containers
            .iter()
            .filter_map(|container| container.id.as_ref())
            .map(|id| id.as_str())
            .collect::<Vec<_>>();

        for id in ids {
            self.daemon
                .remove_container(id, None::<RemoveContainerOptions>)
                .await?;

            tracing::debug!("Successfully removed docker container: {}", id);
        }

        Ok(())
    }

    /// Execute a command on a running container by its ID and return the output.
    pub async fn remote_exec(&self, id: &str, cmd: Vec<&str>) -> Result<Vec<LogOutput>> {
        let exec_options = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(cmd),
            ..Default::default()
        };

        let exec = self.daemon.create_exec(id, exec_options).await?;

        let mut result: Vec<LogOutput> = Vec::new();
        match self.daemon.start_exec(&exec.id, None).await? {
            StartExecResults::Attached { mut output, .. } => {
                while let Some(Ok(msg)) = output.next().await {
                    result.push(msg);
                }
            }
            StartExecResults::Detached => {
                unreachable!("Detached docker exec result is unsupported")
            }
        }

        Ok(result)
    }
}
