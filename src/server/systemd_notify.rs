#[cfg(unix)]
pub fn systemd_notify(server_status_message: &str) {
  {
    use sd_notify::{NotifyState, notify};
    if let Err(e) = notify(&[NotifyState::Ready]) {
      log::error!("Failed to notify systemd: {}", e);
    }

    let _ = notify(&[NotifyState::Status(server_status_message)]);
  }
}
