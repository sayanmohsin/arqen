use std::sync::Arc;

use arqen::thingd::MemoryThingdBackend;

#[path = "common/phase18_contract.rs"]
mod phase18_contract;

#[tokio::test]
async fn memory_backend_passes_phase18_isolation_and_mutation_contract() {
    phase18_contract::run_phase18_contract(Arc::new(MemoryThingdBackend::new())).await;
}

#[cfg(feature = "thingd-native")]
#[tokio::test]
async fn native_backend_passes_phase18_isolation_and_mutation_contract() {
    phase18_contract::run_phase18_contract(Arc::new(arqen::thingd::NativeThingdBackend::memory()))
        .await;
}
