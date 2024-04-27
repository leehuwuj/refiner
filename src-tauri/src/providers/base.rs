pub trait Provider {
    fn completion(&self, prompt: &str) -> impl std::future::Future<Output = String> + Send;
}
