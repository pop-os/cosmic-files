use cctk::{
    cosmic_protocols::{
        toplevel_info::v1::client::zcosmic_toplevel_handle_v1,
        toplevel_management::v1::client::zcosmic_toplevel_manager_v1,
    },
    sctk::{
        output::{OutputHandler, OutputState},
        registry::{ProvidesRegistryState, RegistryState},
        reexports::{calloop, calloop_wayland_source::WaylandSource},
    },
    toplevel_info::{ToplevelInfo, ToplevelInfoHandler, ToplevelInfoState},
    toplevel_management::{ToplevelManagerHandler, ToplevelManagerState},
    wayland_client::{
        globals::registry_queue_init,
        protocol::wl_output,
        Connection, Proxy, QueueHandle, WEnum,
    },
    wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1,
};
use cctk::workspace::{WorkspaceState, WorkspaceHandler};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreKind {
    Normal,
    Maximized,
}

#[derive(Debug, Clone)]
struct SnapshotEntry {
    foreign_id: u32,
    restore_kind: RestoreKind,
    was_active: bool,
}

#[derive(Debug, Clone)]
enum Request {
    Toggle(mpsc::Sender<Result<(), WmError>>),
    Status(mpsc::Sender<String>),
}

#[derive(Debug, Clone)]
pub struct WindowManager {
    request_tx: calloop::channel::Sender<Request>,
}

impl WindowManager {
    pub fn new() -> Result<Self, WmError> {
        let (request_tx, request_rx) = calloop::channel::channel::<Request>();

        thread::Builder::new()
            .name("cosmic-show-desktop-wayland".to_string())
            .spawn(move || {
                let _ = wayland_handler(request_rx);
            })
            .map_err(|e| WmError::Command(format!("failed to spawn wayland thread: {e}")))?;

        Ok(Self { request_tx })
    }

    pub fn status(&self) -> Result<String, WmError> {
        let (tx, rx) = mpsc::channel();
        self.request_tx.send(Request::Status(tx))
            .map_err(|e| WmError::Command(e.to_string()))?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|e| WmError::Command(e.to_string()))
    }

    pub fn toggle(&self) -> Result<(), WmError> {
        let (tx, rx) = mpsc::channel();
        self.request_tx
            .send(Request::Toggle(tx))
            .map_err(|e| WmError::Command(format!("failed to send toggle request: {e}")))?;

        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|e| WmError::Command(format!("toggle response timeout/error: {e}")))?
    }
}

struct AppData {
    exit: bool,
    registry_state: RegistryState,
    output_state: OutputState,
    workspace_state: WorkspaceState,
    toplevel_info_state: ToplevelInfoState,
    toplevel_manager_state: Option<ToplevelManagerState>,
    snapshot: Vec<SnapshotEntry>,
    snapshot_ids: HashSet<u32>,
    supports_move_to_ext_workspace: bool,
}

impl AppData {
    fn handle_toggle(&mut self) -> Result<(), WmError> {
        if self.snapshot.is_empty() {
            self.minimize_all()
        } else {
            self.restore_snapshot()
        }
    }

    fn discard_snapshot(&mut self, _reason: &str) {
        if self.snapshot.is_empty() {
            return;
        }

        self.snapshot.clear();
        self.snapshot_ids.clear();
    }

    fn minimize_all(&mut self) -> Result<(), WmError> {
        let Some(manager) = self.toplevel_manager_state.as_ref() else {
            return Err(WmError::Command(
                "cosmic toplevel manager is unavailable".to_string(),
            ));
        };

        let mut snapshot: Vec<SnapshotEntry> = Vec::new();
        let mut snapshot_ids: HashSet<u32> = HashSet::new();

        let active_workspaces: HashSet<u32> = self.workspace_state.workspaces()
            .filter(|w| w.state.contains(cctk::wayland_protocols::ext::workspace::v1::client::ext_workspace_handle_v1::State::Active))
            .map(|w| w.handle.id().protocol_id()).collect();
        if active_workspaces.is_empty() { return Ok(()); }
        for info in self.toplevel_info_state.toplevels() {
            if !info.workspace.iter().any(|w| active_workspaces.contains(&w.id().protocol_id())) {
                continue;
            }
            if !should_manage_toplevel(info) {
                continue;
            }

            let Some(cosmic_toplevel) = info.cosmic_toplevel.as_ref() else {
                continue;
            };

            let is_minimized = info
                .state
                .contains(&zcosmic_toplevel_handle_v1::State::Minimized);
            if is_minimized {
                continue;
            }

            let restore_kind = if info
                .state
                .contains(&zcosmic_toplevel_handle_v1::State::Maximized)
            {
                RestoreKind::Maximized
            } else {
                RestoreKind::Normal
            };
            let was_active = info
                .state
                .contains(&zcosmic_toplevel_handle_v1::State::Activated);

            let foreign_id = info.foreign_toplevel.id().protocol_id();
            manager.manager.set_minimized(cosmic_toplevel);
            snapshot.push(SnapshotEntry {
                foreign_id,
                restore_kind,
                was_active,
            });
            snapshot_ids.insert(foreign_id);
        }

        self.snapshot = snapshot;
        self.snapshot_ids = snapshot_ids;

        Ok(())
    }

    fn restore_snapshot(&mut self) -> Result<(), WmError> {
        let Some(manager) = self.toplevel_manager_state.as_ref() else {
            return Err(WmError::Command(
                "cosmic toplevel manager is unavailable".to_string(),
            ));
        };

        let snapshot = std::mem::take(&mut self.snapshot);
        self.snapshot_ids.clear();

        let current_toplevels: HashMap<u32, &ToplevelInfo> = self
            .toplevel_info_state
            .toplevels()
            .map(|info| (info.foreign_toplevel.id().protocol_id(), info))
            .collect();

        for entry in snapshot.iter().filter(|entry| !entry.was_active).chain(
            snapshot.iter().filter(|entry| entry.was_active),
        ) {
            let Some(info) = current_toplevels.get(&entry.foreign_id).copied() else {
                continue;
            };

            let Some(cosmic_toplevel) = info.cosmic_toplevel.as_ref() else {
                continue;
            };

            manager.manager.unset_minimized(cosmic_toplevel);
            match entry.restore_kind {
                RestoreKind::Maximized => manager.manager.set_maximized(cosmic_toplevel),
                RestoreKind::Normal => manager.manager.unset_maximized(cosmic_toplevel),
            }
        }

        Ok(())
    }
}

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    cctk::sctk::registry_handlers!(OutputState);
}

impl OutputHandler for AppData {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl ToplevelInfoHandler for AppData {
    fn toplevel_info_state(&mut self) -> &mut ToplevelInfoState {
        &mut self.toplevel_info_state
    }

    fn new_toplevel(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
        if self.snapshot.is_empty() {
            return;
        }

        if let Some(info) = self.toplevel_info_state.info(toplevel) {
            let foreign_id = info.foreign_toplevel.id().protocol_id();
            let should_manage = should_manage_toplevel(info);
            let is_minimized = info
                .state
                .contains(&zcosmic_toplevel_handle_v1::State::Minimized);
            let in_snapshot = self.snapshot_ids.contains(&foreign_id);
            if should_manage && !is_minimized && !in_snapshot {
                let identifier = info.identifier.clone();
                self.discard_snapshot(&format!(
                    "new_toplevel: external window became visible id={foreign_id} identifier={identifier}"
                ));
            }
        }
    }

    fn update_toplevel(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
        if self.snapshot.is_empty() {
            return;
        }

        if let Some(info) = self.toplevel_info_state.info(toplevel) {
            let foreign_id = info.foreign_toplevel.id().protocol_id();
            let should_manage = should_manage_toplevel(info);
            let is_minimized = info
                .state
                .contains(&zcosmic_toplevel_handle_v1::State::Minimized);
            let in_snapshot = self.snapshot_ids.contains(&foreign_id);
            if should_manage && !is_minimized && !in_snapshot {
                let identifier = info.identifier.clone();
                self.discard_snapshot(&format!(
                    "update_toplevel: external window became visible id={foreign_id} identifier={identifier}"
                ));
            }
        }
    }

    fn toplevel_closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
    }
}

impl ToplevelManagerHandler for AppData {
    fn toplevel_manager_state(&mut self) -> &mut ToplevelManagerState {
        self.toplevel_manager_state
            .as_mut()
            .expect("toplevel manager should be initialized")
    }

    fn capabilities(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        capabilities: Vec<WEnum<zcosmic_toplevel_manager_v1::ZcosmicToplelevelManagementCapabilitiesV1>>,
    ) {
        self.supports_move_to_ext_workspace = false;
        for capability in capabilities.iter() {
            if let WEnum::Value(capability) = capability {
                if *capability
                    == zcosmic_toplevel_manager_v1::ZcosmicToplelevelManagementCapabilitiesV1::MoveToExtWorkspace
                {
                    self.supports_move_to_ext_workspace = true;
                }
            }
        }
    }
}

fn should_manage_toplevel_values(identifier: &str, app_id: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }

    if app_id.contains("cosmic-panel")
        || app_id.contains("cosmic-ext-showdesktop")
        || app_id.contains("cosmic-bg")
        || app_id.contains("cosmic-greeter")
    {
        return false;
    }

    true
}

fn should_manage_toplevel(info: &ToplevelInfo) -> bool {
    should_manage_toplevel_values(&info.identifier, &info.app_id)
}

fn wayland_handler(
    request_rx: calloop::channel::Channel<Request>,
) -> Result<(), WmError> {
    let conn = Connection::connect_to_env()
        .map_err(|e| WmError::Command(format!("wayland connect failed: {e}")))?;
    let (globals, event_queue) = registry_queue_init(&conn)
        .map_err(|e| WmError::Command(format!("registry init failed: {e}")))?;
    let qh = event_queue.handle();

    let mut event_loop = calloop::EventLoop::<AppData>::try_new()
        .map_err(|e| WmError::Command(format!("event loop create failed: {e}")))?;

    let wayland_source = WaylandSource::new(conn, event_queue);
    wayland_source
        .insert(event_loop.handle().clone())
        .map_err(|e| WmError::Command(format!("failed to insert wayland source: {e}")))?;

    event_loop
        .handle()
        .insert_source(request_rx, |event, _, state| match event {
            calloop::channel::Event::Msg(Request::Toggle(responder)) => {
                let _ = responder.send(state.handle_toggle());
            }
            calloop::channel::Event::Msg(Request::Status(responder)) => {
                let infos: Vec<_> = state.toplevel_info_state.toplevels()
                    .filter(|i| should_manage_toplevel(i)).collect();
                let minimized = infos.iter().filter(|i| i.state.contains(
                    &zcosmic_toplevel_handle_v1::State::Minimized)).count();
                let _ = responder.send(format!("windows={} minimized={} snapshot={} manager={}",
                    infos.len(), minimized, state.snapshot.len(), state.toplevel_manager_state.is_some()));
            }
            calloop::channel::Event::Closed => {
                state.exit = true;
            }
        })
        .map_err(|e| WmError::Command(format!("failed to add request source: {e}")))?;

    let registry_state = RegistryState::new(&globals);
    let output_state = OutputState::new(&globals, &qh);
    let workspace_state = WorkspaceState::new(&registry_state, &qh);
    let toplevel_info_state = ToplevelInfoState::new(&registry_state, &qh);
    let toplevel_manager_state = ToplevelManagerState::try_new(&registry_state, &qh);

    let mut app_data = AppData {
        exit: false,
        registry_state,
        output_state,
        workspace_state,
        toplevel_info_state,
        toplevel_manager_state,
        snapshot: Vec::new(),
        snapshot_ids: HashSet::new(),
        supports_move_to_ext_workspace: false,
    };

    while !app_data.exit {
        event_loop
            .dispatch(None, &mut app_data)
            .map_err(|e| WmError::Command(format!("event loop dispatch failed: {e}")))?;
    }

    Ok(())
}

cctk::sctk::delegate_output!(AppData);
cctk::sctk::delegate_registry!(AppData);
cctk::delegate_toplevel_info!(AppData);
cctk::delegate_toplevel_manager!(AppData);

#[derive(Debug, Error, Clone)]
pub enum WmError {
    #[error("failed to execute command: {0}")]
    Command(String),
}

#[cfg(test)]
mod tests {
    use super::should_manage_toplevel_values;
    #[test]
    fn skips_shell_surfaces_and_unidentified_windows() {
        assert!(!should_manage_toplevel_values("", "org.example.App"));
        for app in ["cosmic-panel", "cosmic-bg", "cosmic-greeter", "cosmic-ext-showdesktop"] {
            assert!(!should_manage_toplevel_values("window", app));
        }
        assert!(should_manage_toplevel_values("window", "com.system76.CosmicTerm"));
    }
}

impl WorkspaceHandler for AppData {
    fn workspace_state(&mut self) -> &mut WorkspaceState { &mut self.workspace_state }
    fn done(&mut self) {}
}
cctk::delegate_workspace!(AppData);
