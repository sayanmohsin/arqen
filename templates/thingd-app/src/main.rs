use arqen::app::ArqenApp;
use arqen::module::Module;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    arqen::logging::init_logging("info", "pretty");

    ArqenApp::builder()
        .name("{{project_name}}")
        .module(AppModule)
        .build()?
        .start()
        .await
}

struct AppModule;

#[async_trait::async_trait]
impl Module for AppModule {
    fn name(&self) -> &str {
        "app"
    }

    async fn health_check(&self) -> arqen::module::ModuleHealth {
        arqen::module::ModuleHealth::Healthy
    }
}
