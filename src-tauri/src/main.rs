#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Silent background mode used by the Task Scheduler.
    if std::env::args().any(|a| a == "--autoclean") {
        vexora_lib::run_autoclean();
        return;
    }

    // Move old Abu Maryam Cleaner settings/history to the VEXORA profile.
    vexora_lib::util::migrate_old_data();

    // Upgrade to administrator when the user allowed it, then make sure only
    // one VEXORA window is running.
    vexora_lib::elevate_if_needed();
    vexora_lib::util::ensure_single_instance();

    vexora_lib::run();
}
