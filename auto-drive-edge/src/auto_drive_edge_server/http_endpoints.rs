use warp::Filter;

pub async fn autoscript_run() {
    let hi = warp::path("autoscript")
        .and(warp::path("run"))
        .and(warp::path::param::<String>())
        .and(warp::header::optional::<String>("user-agent"))
        .map(|param: String, agent: Option<String>| {
            format!(
                "Hello {}, whose agent is {}",
                param,
                agent.unwrap_or_else(|| "unknown".to_string())
            )
        });
}
