pub mod cache;
pub mod factory;
pub mod memory;
pub mod scoped;
pub mod traits;

#[cfg(feature = "http-client")]
pub mod sync;

#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "thingd-native")]
pub mod native;
#[cfg(feature = "thingd-native")]
pub mod native_backend;

pub use cache::{CachePolicy, CachingThingdBackend};
pub use factory::StorageFactory;
pub use memory::MemoryThingdBackend;
pub use scoped::{ScopeSubject, ScopedThingdBackend, StorageScope};
pub use traits::*;

#[cfg(feature = "http-client")]
pub use sync::{
    ApplyResult, ReplicationChange, ReplicationSnapshot, ReplicationStatus, SyncCheckpointStore,
    SyncClientPolicy, SyncEndpoint, SyncPage, ThingdSyncClient, ThingdSyncWorker,
};

#[cfg(feature = "http-client")]
pub use http::HttpClientPolicy;
#[cfg(feature = "http-client")]
pub use http::HttpThingdBackend;
#[cfg(feature = "thingd-native")]
pub use native::{NativeThingdEngine, NativeThingdStore};
#[cfg(feature = "thingd-native")]
pub use native_backend::NativeThingdBackend;
