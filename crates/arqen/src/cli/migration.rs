use anyhow::Result;

use super::output::Output;
use crate::migration::{NativeToHttpMigrator, ThingdMigrationOptions};

pub fn run(options: ThingdMigrationOptions, check: bool, output: &Output) -> Result<()> {
    let report = tokio::runtime::Runtime::new()?.block_on(async {
        let migrator = NativeToHttpMigrator;
        if check {
            migrator.validate(&options).await
        } else {
            migrator.migrate(&options).await
        }
    })?;
    if output.is_json() {
        output.print_json(serde_json::to_value(report)?);
    } else {
        output.print(&format!(
            "Thingd migration {}: {} objects, {} events, {} jobs, {} indexes{}",
            if check || options.dry_run {
                "checked"
            } else {
                "complete"
            },
            report.objects,
            report.events,
            report.jobs,
            report.indexes,
            report
                .snapshot_path
                .map_or_else(String::new, |path| format!(
                    " (snapshot: {})",
                    path.display()
                ))
        ));
    }
    Ok(())
}
