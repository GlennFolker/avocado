use crate::incl::*;

#[derive(Resource, Default, Clone)]
pub struct TaskPoolConfig {
    pub async_pool: TaskPoolConf,
    pub compute_pool: TaskPoolConf,
    pub io_pool: TaskPoolConf,
}

#[derive(Default, Clone)]
pub struct TaskPoolConf {
    pub threads: Option<usize>,
    pub stack_size: Option<usize>,
    pub thread_name: Option<Cow<'static, str>>,
}
