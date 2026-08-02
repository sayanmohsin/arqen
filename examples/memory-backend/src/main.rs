use arqen::thingd::{MemoryThingdBackend, ThingdBackend};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Memory Backend Example");
    println!("======================\n");
    
    // Create memory backend
    let backend = MemoryThingdBackend::new();
    
    // Seed sample data
    println!("Seeding sample data...");
    backend.seed().await?;
    
    // Object operations
    println!("\n1. Object Operations:");
    let user = backend.put_object("users", "user1", json!({
        "name": "Alice",
        "email": "alice@example.com"
    })).await?;
    println!("   Created user: {} - {}", user.id, user.data["name"]);
    
    let fetched = backend.get_object("users", "user1").await?;
    println!("   Fetched user: {}", fetched.unwrap().data["name"]);
    
    // Query objects
    let users = backend.query_objects("users", None).await?;
    println!("   Total users: {}", users.len());
    
    // Event operations
    println!("\n2. Event Operations:");
    let event = backend.append_event("user_events", "user_created", json!({
        "user_id": "user1"
    })).await?;
    println!("   Appended event: {} - {}", event.id, event.event_type);
    
    let events = backend.read_events("user_events", None, 10).await?;
    println!("   Total events: {}", events.len());
    
    // Job operations
    println!("\n3. Job Operations:");
    let job = backend.push_job("email_queue", json!({
        "to": "alice@example.com",
        "subject": "Welcome"
    }), 3).await?;
    println!("   Pushed job: {} - state: {:?}", job.id, job.state);
    
    let claimed = backend.claim_job("email_queue", "worker1", 30).await?;
    println!("   Claimed job: {}", claimed.unwrap().id);
    
    // Link operations
    println!("\n4. Link Operations:");
    let link = backend.create_link("user1", "post1", "authored").await?;
    println!("   Created link: {} - {}", link.source_id, link.relation);
    
    let links = backend.get_links("user1", None).await?;
    println!("   Total links for user1: {}", links.len());
    
    // Search operations
    println!("\n5. Search Operations:");
    let results = backend.search("Alice", arqen::thingd::traits::SearchOptions {
        limit: 10,
        offset: 0,
        filters: vec![],
    }).await?;
    println!("   Search results for 'Alice': {} items", results.total);
    
    // Reset
    println!("\n6. Reset Operations:");
    backend.reset().await?;
    let count = backend.count_objects("users").await?;
    println!("   After reset, users count: {}", count);
    
    println!("\nExample completed successfully!");
    Ok(())
}
