use std::thread::sleep;
use std::time::Duration;

use kicad_wakatime::{Plugin, traits::DebugProcesses};
// use std::fs;
// use std::process;
use active_win_pos_rs::get_active_window;
use clap::Parser;
use env_logger::Env;
use log::debug;
// use log::error;
use log::info;
use sysinfo::System;

/// WakaTime plugin for KiCAD nightly
#[derive(Parser)]
pub struct Args {
  #[clap(long)]
  debug: bool,
  #[clap(long)]
  disable_heartbeats: bool,
  /// Sleep for 5 seconds after every iteration
  #[clap(long)]
  sleepy: bool,
}

fn main() -> Result<(), anyhow::Error> {
  // pre-initialization
  let args = Args::parse();
  let log_level = match args.debug {
    true => "debug",
    false => "info",
  };
  env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();
  debug!("(os, arch) = {:?}", kicad_wakatime::env_consts());
  let mut sys = System::new_all();
  sys.refresh_all();
  sys.debug_processes();

  // initialization
  info!("Initializing kicad-wakatime...");
  let mut plugin = Plugin::new(
    args.disable_heartbeats,
  );
  plugin.create_file_watcher()?;
  // plugin.file_watcher = Some(watcher);
  plugin.check_cli_installed()?;
  plugin.get_api_key()?;
  plugin.await_connect_to_kicad()?;

  // main loop
  loop {
    debug!("{:?}", get_active_window());
    plugin.set_current_time(plugin.current_time());
    let k = plugin.kicad.as_ref().unwrap();
    let schematic = k.get_open_documents(kicad::DocumentType::DOCTYPE_SCHEMATIC);
    let board = k.get_open_documents(kicad::DocumentType::DOCTYPE_PCB);
    // the KiCAD IPC API does not work properly with schematics as of November 2024
    // (cf. kicad-rs/issues/3), so for the schematic editor, we have to read the
    // focused file raw
    if let Ok(schematic) = schematic {
      let schematic_ds = schematic.first().unwrap();
      debug!("schematic_ds = {:?}", schematic_ds);
      // plugin.set_current_file_from_document_specifier(schematic_ds)?;
      // TODO
    }
    // for the PCB editor, we can instead use the Rust bindings proper
    if let Ok(board) = board {
      let board_ds = board.first().unwrap();
      debug!("board_ds = {:?}", board_ds);
      plugin.set_current_file_from_document_specifier(board_ds.clone())?;
      plugin.set_many_items()?;
    }
    plugin.try_recv()?;
    plugin.first_iteration_finished = true;
    if args.sleepy {
      sleep(Duration::from_secs(5));
    }
  }

  // TODO: this is unreachable
  Ok(())
}