pub trait Adapter {

    async fn query<T, O>(&self, query: &String, params: T) -> O;

}
