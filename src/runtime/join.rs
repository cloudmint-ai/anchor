use crate::*;
use futures::future::try_join_all;
use runtime::JoinHandle;

// 全部执行后，依次返回全部成功结果或者返回第一个错误
// 若 JoinHandler<T> 里的 T 不是 Result 类型，目前需要额外写一层 Ok() 使得其满足Result结构
pub async fn join_all<T>(handlers: Vec<JoinHandle<Result<T>>>) -> Result<Vec<T>> {
    let join_results = try_join_all(handlers).await?;
    let mut results = Vec::with_capacity(join_results.len());
    for join_result in join_results {
        results.push(join_result?)
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use crate::*;

    async fn ok(id: i32) -> Result<i32> {
        Ok(id)
    }

    async fn unexpected() -> Result<i32> {
        Unexpected!("for test")
    }

    #[test::case]
    async fn test_spawn() {
        let mut task_handles = Vec::new();
        for id in vec![1, 2, 3, 4, 5, 6] {
            let handle = runtime::spawn(async move { ok(id).await });
            task_handles.push(handle);
        }
        assert_eq!(
            runtime::join_all(task_handles).await?,
            vec![1, 2, 3, 4, 5, 6]
        );

        let mut task_handles = Vec::new();
        for id in vec![1, 2, 3, 4, 5, 6] {
            let handle = if id == 5 {
                runtime::spawn(async move { unexpected().await })
            } else {
                runtime::spawn(async move { ok(id).await })
            };
            task_handles.push(handle);
        }
        if let Ok(_) = runtime::join_all(task_handles).await {
            panic!("it should err")
        }

        let mut task_handles = Vec::new();
        for id in vec![1, 2, 3, 4, 5, 6] {
            let handle = runtime::spawn(async move { Ok(id) });
            task_handles.push(handle);
        }
        assert_eq!(
            runtime::join_all(task_handles).await?,
            vec![1, 2, 3, 4, 5, 6]
        );
    }
}
