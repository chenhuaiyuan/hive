use async_trait::async_trait;

#[async_trait]
pub trait FileDataTrait {
    fn field_name(&self) -> String;
    fn file_name(&self) -> String;
    fn content_type(&self) -> String;

    async fn save(self, path: Option<String>) -> String;
    async fn new(file_path: String, field_name: String) -> Self;
}
