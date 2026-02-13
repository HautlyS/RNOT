use anyhow::Result;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum StartTrigger {
    Boot,
    Login,
}

pub struct ServiceManager;

impl ServiceManager {
    pub fn install(skip_prompt: bool) -> Result<()> {
        let trigger = if skip_prompt {
            StartTrigger::Boot
        } else {
            Self::prompt_trigger()?
        };

        #[cfg(target_os = "linux")]
        Self::install_systemd(trigger)?;

        #[cfg(target_os = "macos")]
        Self::install_launchd(trigger)?;

        #[cfg(target_os = "windows")]
        Self::install_windows_service(trigger)?;

        Ok(())
    }

    fn prompt_trigger() -> Result<StartTrigger> {
        println!("RNOT Service Installation");
        println!("{:-<40}", "");
        println!("When should RNOT start?");
        println!("  1) On system boot (requires system service)");
        println!("  2) On user login (user service, recommended)");
        println!();
        print!("Select [1-2] (default: 2): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        let trigger = match choice {
            "1" => StartTrigger::Boot,
            "" | "2" => StartTrigger::Login,
            _ => {
                println!("Invalid choice, using default (login)");
                StartTrigger::Login
            }
        };

        println!();
        print!("Install RNOT service? [y/N]: ");
        io::stdout().flush()?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

        if !confirm.trim().eq_ignore_ascii_case("y") {
            anyhow::bail!("Installation cancelled");
        }

        Ok(trigger)
    }

    pub fn uninstall() -> Result<()> {
        #[cfg(target_os = "linux")]
        Self::uninstall_systemd()?;

        #[cfg(target_os = "macos")]
        Self::uninstall_launchd()?;

        #[cfg(target_os = "windows")]
        Self::uninstall_windows_service()?;

        Ok(())
    }

    pub fn status() -> Result<()> {
        #[cfg(target_os = "linux")]
        Self::status_systemd()?;

        #[cfg(target_os = "macos")]
        Self::status_launchd()?;

        #[cfg(target_os = "windows")]
        Self::status_windows_service()?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn install_systemd(trigger: StartTrigger) -> Result<()> {
        let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let binary_path = Self::get_binary_path()?;

        match trigger {
            StartTrigger::Login => {
                let service_content = format!(
                    r#"[Unit]
Description=RNOT Website Monitor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={binary} daemon
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
"#,
                    binary = binary_path.display()
                );

                let service_dir = PathBuf::from(&home).join(".config/systemd/user");
                fs::create_dir_all(&service_dir)?;

                let service_file = service_dir.join("rnot.service");
                fs::write(&service_file, service_content)?;

                println!("Installing systemd user service...");

                Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .status()?;

                Command::new("systemctl")
                    .args(["--user", "enable", "rnot.service"])
                    .status()?;

                Command::new("systemctl")
                    .args(["--user", "start", "rnot.service"])
                    .status()?;

                let _ = Command::new("loginctl")
                    .args(["enable-linger", &user])
                    .status();

                println!("✓ Service installed and started successfully!");
                println!("  Type: User service (starts on login)");
                println!("\nUseful commands:");
                println!("  systemctl --user status rnot    # Check status");
                println!("  systemctl --user stop rnot      # Stop service");
                println!("  systemctl --user restart rnot   # Restart service");
                println!("  journalctl --user -u rnot -f    # View logs");
            }
            StartTrigger::Boot => {
                let service_content = format!(
                    r#"[Unit]
Description=RNOT Website Monitor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={user}
WorkingDirectory={home}
ExecStart={binary} daemon
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
"#,
                    user = user,
                    home = home,
                    binary = binary_path.display()
                );

                let service_file = PathBuf::from("/tmp/rnot.service");
                fs::write(&service_file, service_content)?;

                println!("Installing system service (requires sudo)...");

                let status = Command::new("sudo")
                    .args([
                        "cp",
                        "/tmp/rnot.service",
                        "/etc/systemd/system/rnot.service",
                    ])
                    .status()?;

                if !status.success() {
                    anyhow::bail!("Failed to copy service file. Do you have sudo access?");
                }

                Command::new("sudo")
                    .args(["systemctl", "daemon-reload"])
                    .status()?;

                Command::new("sudo")
                    .args(["systemctl", "enable", "rnot.service"])
                    .status()?;

                Command::new("sudo")
                    .args(["systemctl", "start", "rnot.service"])
                    .status()?;

                fs::remove_file(&service_file)?;

                println!("✓ Service installed and started successfully!");
                println!("  Type: System service (starts on boot)");
                println!("\nUseful commands:");
                println!("  sudo systemctl status rnot    # Check status");
                println!("  sudo systemctl stop rnot      # Stop service");
                println!("  sudo systemctl restart rnot   # Restart service");
                println!("  sudo journalctl -u rnot -f    # View logs");
            }
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn uninstall_systemd() -> Result<()> {
        println!("Uninstalling systemd service...");

        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let user_service = PathBuf::from(&home).join(".config/systemd/user/rnot.service");
        let system_service = PathBuf::from("/etc/systemd/system/rnot.service");

        if user_service.exists() {
            let _ = Command::new("systemctl")
                .args(["--user", "stop", "rnot.service"])
                .status();

            let _ = Command::new("systemctl")
                .args(["--user", "disable", "rnot.service"])
                .status();

            fs::remove_file(&user_service)?;

            let _ = Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status();

            println!("✓ User service uninstalled successfully!");
        } else if system_service.exists() {
            let _ = Command::new("sudo")
                .args(["systemctl", "stop", "rnot.service"])
                .status();

            let _ = Command::new("sudo")
                .args(["systemctl", "disable", "rnot.service"])
                .status();

            let status = Command::new("sudo")
                .args(["rm", "/etc/systemd/system/rnot.service"])
                .status()?;

            if status.success() {
                let _ = Command::new("sudo")
                    .args(["systemctl", "daemon-reload"])
                    .status();

                println!("✓ System service uninstalled successfully!");
            } else {
                anyhow::bail!("Failed to remove service file. Do you have sudo access?");
            }
        } else {
            println!("No RNOT service found");
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn status_systemd() -> Result<()> {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let user_service = PathBuf::from(&home).join(".config/systemd/user/rnot.service");

        if user_service.exists() {
            let output = Command::new("systemctl")
                .args(["--user", "status", "rnot.service"])
                .output()?;

            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            let output = Command::new("systemctl")
                .args(["status", "rnot.service"])
                .output()?;

            println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn install_launchd(trigger: StartTrigger) -> Result<()> {
        let home = env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
        let binary_path = Self::get_binary_path()?;

        let run_at_load = matches!(trigger, StartTrigger::Boot);

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rnot.monitor</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <{run_at_load}/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{home}/Library/Logs/rnot.log</string>
    <key>StandardErrorPath</key>
    <string>{home}/Library/Logs/rnot.error.log</string>
    <key>WorkingDirectory</key>
    <string>{home}</string>
</dict>
</plist>
"#,
            binary = binary_path.display(),
            home = home,
            run_at_load = if run_at_load { "true" } else { "false" }
        );

        let launch_agents_dir = PathBuf::from(&home).join("Library/LaunchAgents");
        fs::create_dir_all(&launch_agents_dir)?;

        let plist_file = launch_agents_dir.join("com.rnot.monitor.plist");
        fs::write(&plist_file, plist_content)?;

        println!("Installing LaunchAgent...");

        Command::new("launchctl")
            .args(["load", plist_file.to_str().unwrap()])
            .status()?;

        println!("✓ Service installed and started successfully!");
        println!(
            "  Type: {} service",
            match trigger {
                StartTrigger::Boot => "Boot",
                StartTrigger::Login => "Login",
            }
        );
        println!("\nUseful commands:");
        println!("  launchctl list | grep rnot           # Check status");
        println!("  launchctl stop com.rnot.monitor      # Stop service");
        println!("  launchctl start com.rnot.monitor     # Start service");
        println!("  tail -f ~/Library/Logs/rnot.log      # View logs");

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn uninstall_launchd() -> Result<()> {
        let home = env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
        let plist_file = PathBuf::from(&home).join("Library/LaunchAgents/com.rnot.monitor.plist");

        println!("Uninstalling LaunchAgent...");

        if plist_file.exists() {
            let _ = Command::new("launchctl")
                .args(["unload", plist_file.to_str().unwrap()])
                .status();

            fs::remove_file(&plist_file)?;
        }

        println!("✓ Service uninstalled successfully!");

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn status_launchd() -> Result<()> {
        let output = Command::new("launchctl").args(["list"]).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("com.rnot.monitor") {
            println!("✓ RNOT service is running");
            for line in stdout.lines() {
                if line.contains("com.rnot.monitor") {
                    println!("{}", line);
                }
            }
        } else {
            println!("✗ RNOT service is not running");
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn install_windows_service(trigger: StartTrigger) -> Result<()> {
        let binary_path = Self::get_binary_path()?;

        println!("Installing Windows service...");
        println!("Note: This requires administrator privileges.");

        let task_name = "RNOT-Monitor";
        let trigger_xml = match trigger {
            StartTrigger::Boot => {
                r#"  <Triggers>
    <BootTrigger>
      <Enabled>true</Enabled>
    </BootTrigger>
  </Triggers>"#
            }
            StartTrigger::Login => {
                r#"  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
    </LogonTrigger>
  </Triggers>"#
            }
        };

        let task_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>RNOT Website Monitor Service</Description>
  </RegistrationInfo>
{trigger_xml}
  <Principals>
    <Principal>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>true</RunOnlyIfNetworkAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions>
    <Exec>
      <Command>{binary}</Command>
      <Arguments>daemon</Arguments>
    </Exec>
  </Actions>
</Task>
"#,
            binary = binary_path.display(),
            trigger_xml = trigger_xml
        );

        let temp_dir = env::temp_dir();
        let task_file = temp_dir.join("rnot-task.xml");
        fs::write(&task_file, task_xml)?;

        let status = Command::new("schtasks")
            .args([
                "/Create",
                "/TN",
                task_name,
                "/XML",
                task_file.to_str().unwrap(),
                "/F",
            ])
            .status()?;

        fs::remove_file(&task_file)?;

        if status.success() {
            println!("✓ Service installed successfully!");
            println!(
                "  Type: {} trigger",
                match trigger {
                    StartTrigger::Boot => "Boot",
                    StartTrigger::Login => "Login",
                }
            );
            println!("\nUseful commands:");
            println!("  schtasks /Query /TN RNOT-Monitor         # Check status");
            println!("  schtasks /Run /TN RNOT-Monitor           # Start service");
            println!("  schtasks /End /TN RNOT-Monitor           # Stop service");
        } else {
            anyhow::bail!("Failed to install service. Try running as administrator.");
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn uninstall_windows_service() -> Result<()> {
        println!("Uninstalling Windows service...");

        let status = Command::new("schtasks")
            .args(["/Delete", "/TN", "RNOT-Monitor", "/F"])
            .status()?;

        if status.success() {
            println!("✓ Service uninstalled successfully!");
        } else {
            println!("✗ Failed to uninstall service. It may not be installed.");
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn status_windows_service() -> Result<()> {
        let output = Command::new("schtasks")
            .args(["/Query", "/TN", "RNOT-Monitor", "/FO", "LIST", "/V"])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("✗ RNOT service is not installed");
        }

        Ok(())
    }

    fn get_binary_path() -> Result<PathBuf> {
        let exe = env::current_exe()?;
        Ok(exe)
    }
}
