use async_executor::LocalExecutor;

thread_local! {
    pub(crate) static LOCAL_EXECUTOR: LocalExecutor<'static> = LocalExecutor::new();
}
