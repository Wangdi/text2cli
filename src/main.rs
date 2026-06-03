use clap::{Parser, Subcommand};
use text2cli::{
    Config, ConfigLoader, ContextCollector, AgentExecutor, AgentRegistry,
    ClaudeAdapter, GenericAdapter, PwshHook, BashHook, ZshHook, ShellHook,
    TriggerParser, SessionManager,
};

#[derive(Parser)]
#[command(name = "text2cli")]
#[command(about = "AI-powered command suggestion CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input to process (if no subcommand)
    #[arg(trailing_var_arg = true)]
    input: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize shell integration
    Init {
        /// Shell type: bash, zsh, pwsh
        shell: String,
    },

    /// Process input and return command
    Process {
        /// Input to process
        #[arg(trailing_var_arg = true)]
        input: Vec<String>,
    },

    /// List available agents
    ListAgents,

    /// Show configuration
    Config,

    /// Session management
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Start a new session
    Start {
        /// Session name
        name: Option<String>,
        /// Agent to use
        #[arg(short, long, default_value = "claude-code")]
        agent: String,
    },

    /// Resume an existing session
    Resume {
        /// Session ID or name
        session: String,
    },

    /// List all sessions
    List,

    /// End current session
    End,

    /// Delete a session
    Delete {
        /// Session ID
        session: String,
    },

    /// Show session history
    History {
        /// Number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = ConfigLoader::load().unwrap_or_else(|e| {
        eprintln!("[text2cli] Warning: {}", e);
        Config::default()
    });

    match cli.command {
        Some(Commands::Init { shell }) => {
            handle_init(&shell);
        }
        Some(Commands::Process { input }) => {
            handle_process(&input, &config).await;
        }
        Some(Commands::ListAgents) => {
            handle_list_agents(&config);
        }
        Some(Commands::Config) => {
            handle_config();
        }
        Some(Commands::Session { command }) => {
            handle_session(command, &config).await;
        }
        None => {
            if !cli.input.is_empty() {
                handle_process(&cli.input, &config).await;
            } else {
                println!("text2cli - AI-powered command suggestion CLI");
                println!("Run 'text2cli --help' for usage.");
            }
        }
    }
}

fn handle_init(shell: &str) {
    match shell {
        "bash" => {
            let hook = BashHook::new("text2cli");
            println!("{}", hook.generate());
        }
        "zsh" => {
            let hook = ZshHook::new("text2cli");
            println!("{}", hook.generate());
        }
        "pwsh" | "powershell" => {
            let hook = PwshHook::new("text2cli");
            println!("{}", hook.generate());
        }
        _ => {
            eprintln!("[text2cli] Unknown shell: {}", shell);
            eprintln!("Supported shells: bash, zsh, pwsh");
            std::process::exit(1);
        }
    }
}

async fn handle_process(input: &[String], config: &Config) {
    let input_str = input.join(" ");
    let parser = TriggerParser::new(&config.trigger);

    let parsed = match parser.parse(&input_str) {
        Ok(Some(cmd)) => cmd,
        Ok(None) => {
            // No trigger found, pass through
            println!("{}", input_str);
            return;
        }
        Err(e) => {
            eprintln!("[text2cli] Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let context = match ContextCollector::collect() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("[text2cli] Context error: {}", e);
            std::process::exit(1);
        }
    };

    let mut registry = AgentRegistry::default();
    registry.register(Box::new(GenericAdapter::new("gemini", "gemini")));
    registry.register(Box::new(GenericAdapter::new("openclaw", "openclaw")));
    registry.register(Box::new(GenericAdapter::new("hermes", "hermes")));

    let agent = registry.get(&config.default_agent).unwrap_or_else(|| {
        eprintln!("[text2cli] Agent '{}' not found", config.default_agent);
        std::process::exit(1);
    });

    let executor = AgentExecutor::new(
        Box::new(ClaudeAdapter::new(agent.command())),
        context,
    );

    match executor.execute(&parsed.content).await {
        Ok(commands) => {
            // Output first command for injection
            println!("{}", commands[0]);
        }
        Err(e) => {
            eprintln!("[text2cli] Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_list_agents(config: &Config) {
    println!("Available agents:");
    for (name, agent_config) in &config.agents {
        let status = if agent_config.enabled { "enabled" } else { "disabled" };
        println!("  {} ({}) - {}", name, agent_config.command, status);
    }
}

fn handle_config() {
    match ConfigLoader::config_path() {
        Ok(path) => {
            println!("Config path: {}", path.display());
            match ConfigLoader::load() {
                Ok(config) => {
                    println!("Trigger: {}", config.trigger);
                    println!("Default agent: {}", config.default_agent);
                }
                Err(e) => {
                    println!("Using defaults: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[text2cli] {}", e);
        }
    }
}

async fn handle_session(command: SessionCommands, _config: &Config) {
    let mut manager = match SessionManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[text2cli] Failed to initialize session manager: {}", e);
            std::process::exit(1);
        }
    };

    match command {
        SessionCommands::Start { name, agent } => {
            match manager.create(name, agent.clone()) {
                Ok(session) => {
                    println!("Session started: {}", session.id);
                    if let Some(name) = &session.name {
                        println!("Name: {}", name);
                    }
                    println!("Agent: {}", agent);
                    println!("\nUse 'text2cli session resume {}' to continue", session.id);
                }
                Err(e) => {
                    eprintln!("[text2cli] Failed to create session: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionCommands::Resume { session } => {
            match manager.load(&session) {
                Ok(session) => {
                    println!("Resumed session: {}", session.id);
                    if let Some(name) = &session.name {
                        println!("Name: {}", name);
                    }
                    println!("Agent: {}", session.agent_name);
                    println!("Commands in history: {}", session.history.len());

                    // Save the session ID for shell integration
                    println!("\nSession ID: {}", session.id);
                }
                Err(e) => {
                    eprintln!("[text2cli] Failed to resume session: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionCommands::List => {
            match manager.list() {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        println!("No sessions found.");
                    } else {
                        println!("Sessions:");
                        for session in sessions {
                            let name = session.name.unwrap_or_else(|| "(unnamed)".to_string());
                            println!(
                                "  {} - {} ({} commands, {})",
                                session.id,
                                name,
                                session.command_count,
                                session.agent_name
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[text2cli] Failed to list sessions: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionCommands::End => {
            manager.end();
            println!("Session ended.");
        }
        SessionCommands::Delete { session } => {
            match manager.delete(&session) {
                Ok(()) => {
                    println!("Session deleted: {}", session);
                }
                Err(e) => {
                    eprintln!("[text2cli] Failed to delete session: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SessionCommands::History { limit } => {
            match manager.current() {
                Some(session) => {
                    let recent = session.recent_commands(limit);
                    if recent.is_empty() {
                        println!("No history in current session.");
                    } else {
                        println!("Session history:");
                        for entry in recent {
                            println!("  {} -> {}",
                                entry.request,
                                entry.commands.join("; ")
                            );
                        }
                    }
                }
                None => {
                    println!("No active session. Use 'text2cli session start' or 'resume'.");
                }
            }
        }
    }
}
