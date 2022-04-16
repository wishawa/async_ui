use async_executor::LocalExecutor;

thread_local! {
    pub(crate) static LOCAL_EXECUTOR: LocalExecutor<'static> = {
        let exe = LocalExecutor::new();
        exe
    };
}
