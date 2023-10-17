use eyre::Result;
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
