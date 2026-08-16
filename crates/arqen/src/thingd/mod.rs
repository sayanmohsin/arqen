pub mod bootstrap;
pub mod cache;
pub mod factory;
pub mod memory;
#[cfg(feature = "thingd-native")]
pub mod native;
#[cfg(feature = "thingd-native")]
pub mod native_backend;
pub mod scoped;
pub mod traits;

#[cfg(feature = "http-client")]
pub mod sync;

#[cfg(feature = "http-client")]
pub mod http;

pub use bootstrap::{BootstrapPolicy, retry as retry_bootstrap, seed_with_retry};
pub use cache::{CachePolicy, CachingThingdBackend};
pub use factory::StorageFactory;
pub use memory::MemoryThingdBackend;
#[cfg(feature = "thingd-native")]
pub use native::{NativeThingdEngine, NativeThingdStore};
#[cfg(feature = "thingd-native")]
pub use native_backend::NativeThingdBackend;
pub use scoped::{ScopeSubject, ScopedThingdBackend, StorageScope};
pub use traits::*;

#[cfg(feature = "http-client")]
pub use sync::{
    ApplyResult, FileSyncCheckpointStore, ReplicationChange, ReplicationSnapshot,
    ReplicationStatus, SyncCheckpointStore, SyncClientPolicy, SyncEndpoint, SyncPage,
    SyncRuntimeStatus, ThingdSyncClient, ThingdSyncWorker,
};

#[cfg(feature = "http-client")]
pub use http::HttpThingdBackend;
#[cfg(feature = "http-client")]
pub use http::{HttpClientPolicy, THINGD_HTTP_API_VERSION};
