#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::{collections::HashMap, fmt::Debug, path::Path};

use bollard::{
    container::{
        CreateContainerOptions, ListContainersOptions, LogOutput, NetworkingConfig,
        RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    },
    exec::{CreateExecOptions, StartExecResults},
    image::BuildImageOptions,
    network::{CreateNetworkOptions, ListNetworksOptions},
    service::{ContainerCreateResponse, ContainerSummary, EndpointSettings, Volume},
    Docker,
};
use eyre::{bail, Result};
use futures_util::{StreamExt, TryStreamExt};
use serde::Serialize;

pub use bollard::container::Config;
pub use bollard::image::CreateImageOptions;
pub use bollard::service::HostConfig;
pub use bollard::volume::CreateVolumeOptions;
pub use build_context::BuildContext;

/// Utilities for building Docker images
mod build_context;

/// The default Docker network name.
pub const DEFAULT_NETWORK_NAME: &str = "opup-net";

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

        tracing::debug!(target: "composer", "Successfully connected to Docker daemon");
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

    /// Create the default Docker network for OP-UP components.
    pub async fn create_default_network(&self) -> Result<()> {
        self.create_network(CreateNetworkOptions {
            name: DEFAULT_NETWORK_NAME,
            ..Default::default()
        })
        .await
    }

    /// Create a Docker network with the specified configs.
    ///
    /// NOTE: This method will overwrite any existing network with the same name.
    pub async fn create_network(&self, mut config: CreateNetworkOptions<&str>) -> Result<()> {
        let existing_networks = self
            .daemon
            .list_networks(None::<ListNetworksOptions<&str>>)
            .await?;

        if existing_networks
            .iter()
            .any(|network| network.name == Some(config.name.to_string()))
        {
            tracing::debug!(target: "composer", "Network {} already exists. Replacing it.", config.name);
            self.daemon.remove_network(config.name).await?;
        }

        config.labels.insert("com.docker.compose.project", "op-up");
        let network = self.daemon.create_network(config).await?;

        tracing::debug!(target: "composer", "Created docker network: {:?}", network);
        Ok(())
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
            .map(|res| {
                res.map(|info| {
                    tracing::trace!(target: "composer", "image progress: {:?}", info);
                    info
                })
            })
            .try_collect::<Vec<_>>()
            .await?;

        tracing::debug!(target: "composer", "Created docker image: {:?}", res);

        match res.first() {
            Some(info) => match info.id.as_ref() {
                Some(id) => Ok(id.clone()),
                None => bail!("No image ID found in response"),
            },
            None => bail!("No image info found in response"),
        }
    }

    /// Build a Docker image from the specified Dockerfile and build context files.
    pub async fn build_image(
        &self,
        name: impl Into<String>,
        build_context: BuildContext<impl AsRef<Path>>,
    ) -> Result<()> {
        let build_options = BuildImageOptions {
            t: name.into(),
            dockerfile: "Dockerfile".to_string(),
            buildargs: build_context.buildargs.clone(),
            pull: true,
            ..Default::default()
        };

        let build_context = build_context.create_archive()?;
        let mut image_build_stream =
            self.daemon
                .build_image(build_options, None, Some(build_context.into()));

        while let Some(build_info) = image_build_stream.next().await {
            let res = match build_info {
                Ok(build_info) => build_info,
                Err(e) => eyre::bail!("Error building docker image: {:?}", e),
            };
            tracing::debug!(target: "composer", "Build info: {:?}", res);
        }

        Ok(())
    }

    /// Creates a Docker volume with the specified options.
    pub async fn create_volume<T>(&self, config: CreateVolumeOptions<T>) -> Result<Volume>
    where
        T: Into<String> + Serialize + Eq + std::hash::Hash,
    {
        self.daemon.create_volume(config).await.map_err(Into::into)
    }

    /// Create a Docker container for the specified OP Stack component
    pub async fn create_container(
        &self,
        name: &str,
        mut config: Config<String>,
        overwrite: bool,
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

        // Check if a container already exists with the specified name. If it does:
        // - If overwrite is true, remove the existing container and create a new one.
        // - If overwrite is false, return the existing container ID.
        let containers = self.list_containers(None).await?;
        if let Some(container) = containers.iter().find(|container| {
            container
                .names
                .as_ref()
                .map(|names| {
                    names
                        .iter()
                        .any(|n| n == name || n == &format!("/{}", name))
                })
                .unwrap_or(false)
        }) {
            tracing::debug!(target: "composer", "Container {} already exists", name);
            let id = container
                .id
                .clone()
                .ok_or_else(|| eyre::eyre!("No container ID found"))?;

            if overwrite {
                self.daemon
                    .remove_container(name, None::<RemoveContainerOptions>)
                    .await?;
                tracing::debug!(target: "composer", "Removed existing docker container {}", name);
            } else {
                return Ok(ContainerCreateResponse {
                    id,
                    warnings: vec![],
                });
            }
        }

        // Add the container to the default network.
        config
            .networking_config
            .get_or_insert(NetworkingConfig {
                endpoints_config: HashMap::new(),
            })
            .endpoints_config
            .insert(
                DEFAULT_NETWORK_NAME.to_string(),
                EndpointSettings::default(),
            );

        let res = self
            .daemon
            .create_container(Some(create_options), config)
            .await?;

        tracing::debug!(target: "composer", "Created docker container {} with ID: {}", name, res.id);

        Ok(res)
    }

    /// Start the specified OP Stack component container by ID.
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.daemon
            .start_container(id, None::<StartContainerOptions<&str>>)
            .await?;

        tracing::debug!(target: "composer", "Started docker container with ID: {}", id);
        Ok(())
    }

    /// Stop the specified OP Stack component container by ID.
    pub async fn stop_container(&self, id: &str) -> Result<()> {
        self.daemon
            .stop_container(id, None::<StopContainerOptions>)
            .await?;

        tracing::debug!(target: "composer", "Stopped docker container with ID: {}", id);
        Ok(())
    }

    /// Remove the specified OP Stack component container by ID.
    pub async fn remove_container(&self, id: &str) -> Result<()> {
        self.daemon
            .remove_container(id, None::<RemoveContainerOptions>)
            .await?;

        tracing::debug!(target: "composer", "Removed docker container with ID: {}", id);
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

        tracing::info!(target: "composer", "Stopping docker containers: {:?}", ids);

        for id in ids {
            self.daemon
                .stop_container(id, None::<StopContainerOptions>)
                .await?;

            tracing::debug!(target: "composer", "Successfully stopped docker container: {}", id);
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

            tracing::debug!(target: "composer", "Successfully removed docker container: {}", id);
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

        match self.daemon.start_exec(&exec.id, None).await? {
            StartExecResults::Attached { output, .. } => Ok(output
                .filter_map(|res| async {
                    match res {
                        Ok(output) => Some(output),
                        Err(e) => {
                            tracing::error!(target: "composer", "Error executing remote command: {:?}", e);
                            None
                        },
                    }
                })
                .collect::<Vec<_>>()
                .await),

            StartExecResults::Detached => {
                bail!("Detached exec is not supported")
            }
        }
    }
}

/// Given a host port, bind it to the container.
pub fn bind_host_port(host_port: u16) -> Option<Vec<bollard::service::PortBinding>> {
    Some(vec![bollard::service::PortBinding {
        host_ip: None,
        host_port: Some(host_port.to_string()),
    }])
}
