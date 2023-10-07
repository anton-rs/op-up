use bollard::image::CreateImageOptions;
use op_composer::Composer;

#[tokio::test]
pub async fn test_basic_docker_composer() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let composer = Composer::new().await;

    let container_name = "hello-world-container";
    let container_image = "hello-world:linux";

    // 1. Pull image from docker hub
    composer
        .create_image(CreateImageOptions {
            from_image: container_image,
            ..Default::default()
        })
        .await?;

    // 2. Create container from image
    let hello_world_container = composer
        .create_container(container_name, container_image)
        .await?;

    let all_containers = composer.list_containers(None).await?;
    assert_eq!(all_containers.len(), 1);

    // 3. Start running container
    composer.start_container(&hello_world_container.id).await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 4. Stop running container
    composer.stop_container(&hello_world_container.id).await?;

    // 5. Remove container artifacts
    composer.remove_container(&hello_world_container.id).await?;

    let all_containers = composer.list_containers(None).await?;
    assert_eq!(all_containers.len(), 0);

    Ok(())
}
