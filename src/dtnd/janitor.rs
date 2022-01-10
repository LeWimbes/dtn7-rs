use log::debug;

use crate::utils::CONFIG;

async fn janitor() {
    debug!("running janitor");

    debug!("cleaning up peers");
    crate::core::process_peers();

    debug!("reprocessing bundles");
    crate::core::process_bundles().await;
}

pub fn spawn_janitor() {
    tokio::spawn(crate::dtnd::cron::spawn_timer(
        (*CONFIG.lock().unwrap()).janitor_interval,
        janitor,
    ));
}
