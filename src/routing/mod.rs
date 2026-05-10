use anyhow::Result;
use matchit::Router;

fn router_gen() -> Result<Router<i32>> {
    let mut router = Router::new();
    router.insert("/users/{id}", 42)?;
    let matched = router.at("/users/1")?;
    return Ok(router);
}
