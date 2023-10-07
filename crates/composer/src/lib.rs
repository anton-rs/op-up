use std::collections::HashMap;

use bollard::{
    container::{
        Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
        StartContainerOptions, StopContainerOptions,
    },
    service::ContainerSummary,
    Docker,
};
use eyre::Result;

/// Placeholder config struct.
/// TODO: Use the actual parsed component TOML file struct instead of this placeholder
pub struct ComponentConfig<'a> {
    name: &'a str,
}

/// The Composer is responsible for managing the OP-UP docker containers.
pub struct Composer {
    pub daemon: Docker,
}

impl Composer {
    /// Create a new instance of the Composer.
    pub async fn new() -> Self {
        let daemon = Docker::connect_with_local_defaults().expect(
            "Failed to connect to Docker daemon. 
            Please check that Docker is installed and running on your machine",
        );

        Self { daemon }
    }

    /// List all the OP-UP docker containers existing on the host.
    ///
    /// The containers are filtered by the label `com.docker.compose.project=op-up`.
    ///
    /// This method allows optional filtering by container status:
    /// `created`|`restarting`|`running`|`removing`|`paused`|`exited`|`dead`
    pub async fn list_opup_containers(
        &self,
        status: Option<&str>,
    ) -> Result<Vec<ContainerSummary>> {
        let mut filters = HashMap::new();
        filters.insert("label", vec!["com.docker.compose.project=op-up"]);

        if let Some(status) = status {
            filters.insert("status", vec![status]);
        }

        let list_options = ListContainersOptions {
            all: true,
            limit: None,
            size: false,
            filters,
        };

        self.daemon
            .list_containers(Some(list_options))
            .await
            .map_err(Into::into)
    }

    /// Create a Docker container for the specified OP Stack component
    ///
    /// The container will be created from the options specified in the component TOML file.
    pub async fn create_container(&self, component: ComponentConfig<'_>) -> Result<()> {
        let create_options = CreateContainerOptions {
            name: component.name,
            platform: None,
        };

        let mut labels = HashMap::new();
        labels.insert("com.docker.compose.project", "op-up");

        let config = Config {
            image: None,
            labels: Some(labels),
            ..Default::default()
        };

        let res = self
            .daemon
            .create_container(Some(create_options), config)
            .await?;

        tracing::debug!(
            "Created docker container {} with ID: {}",
            component.name,
            res.id
        );

        Ok(())
    }

    /// Start the specified OP Stack component container.
    pub async fn start_container(&self, name: &str) -> Result<()> {
        self.daemon
            .start_container(name, None::<StartContainerOptions<&str>>)
            .await?;

        Ok(())
    }

    /// Stop all OP-UP docker containers at once.
    pub async fn stop_all_opup_containers(&self) -> Result<()> {
        let running_containers = self.list_opup_containers(Some("running")).await?;

        let names = running_containers
            .iter()
            .filter_map(|container| container.names.as_ref().and_then(|names| names.first()))
            .collect::<Vec<_>>();

        tracing::info!("Stopping containers: {:?}", names);

        for name in names {
            self.daemon
                .stop_container(name, None::<StopContainerOptions>)
                .await?;

            tracing::debug!("Successfully stopped container: {}", name);
        }

        Ok(())
    }

    pub async fn purge_all_opup_containers(&self) -> Result<()> {
        let containers = self.list_opup_containers(None).await?;

        for container in containers {
            let name = container
                .names
                .as_ref()
                .and_then(|names| names.first())
                .ok_or_else(|| eyre::eyre!("Container name not found"))?;

            self.daemon
                .remove_container(name, None::<RemoveContainerOptions>)
                .await?;
        }

        Ok(())
    }
}
