use anyhow::Result;
use std::future::Future;

/// Creates a new fully-featured tokio multi-thread [Runtime](tokio::runtime::Runtime).
pub fn tokio_runtime() -> Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()?)
}

/// Executes a regular future until completion or until external signal received.
pub fn run_until_ctrl_c<F>(fut: F) -> Result<()>
where
    F: Future<Output = Result<()>>,
{
    let tokio_runtime = tokio_runtime()?;
    tokio_runtime.block_on(fut)?;
    Ok(())
}

/// Run a future until ctrl-c is pressed.
pub fn run_blocking_until_ctrl_c<F>(fut: F) -> Result<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    let tokio_runtime = tokio_runtime()?;
    let handle = tokio_runtime.handle().clone();
    let fut = tokio_runtime
        .handle()
        .spawn_blocking(move || handle.block_on(fut));
    tokio_runtime.block_on(async move { fut.await.expect("join task") })?;
    std::thread::spawn(move || drop(tokio_runtime));
    Ok(())
}
