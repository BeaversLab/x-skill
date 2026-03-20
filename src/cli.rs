use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "x-skill",
    about = "The open agent skills ecosystem",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install skills from a source
    #[command(visible_aliases = ["a", "i", "install"])]
    Add {
        /// Source to install from (GitHub shorthand, URL, local path, etc.)
        source: String,

        /// Install to global (home) directory
        #[arg(short, long)]
        global: bool,

        /// Skip all prompts
        #[arg(short, long)]
        yes: bool,

        /// List discovered skills and exit
        #[arg(short, long)]
        list: bool,

        /// Select all skills and all agents
        #[arg(long)]
        all: bool,

        /// Target agent(s). Use '*' for all.
        #[arg(short, long, num_args = 1..)]
        agent: Vec<String>,

        /// Target skill(s). Use '*' for all.
        #[arg(short, long, num_args = 1..)]
        skill: Vec<String>,

        /// Enable full-depth recursive skill discovery
        #[arg(long)]
        full_depth: bool,

        /// Force copy mode instead of symlink
        #[arg(long)]
        copy: bool,
    },

    /// Remove installed skills
    #[command(visible_aliases = ["rm", "r"])]
    Remove {
        /// Skill name to remove
        skill: Option<String>,

        /// Target agent
        #[arg(short, long)]
        agent: Option<String>,

        /// Remove from global directory
        #[arg(short, long)]
        global: bool,

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// List installed skills
    #[command(visible_alias = "ls")]
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// List global skills
        #[arg(short, long)]
        global: bool,
    },

    /// Search for skills
    #[command(visible_aliases = ["search", "f", "s"])]
    Find {
        /// Search query
        query: Option<String>,
    },

    /// Check for skill updates
    Check,

    /// Update all outdated skills
    #[command(visible_alias = "upgrade")]
    Update,

    /// Create a new SKILL.md template
    Init {
        /// Skill name (defaults to current directory name)
        name: Option<String>,
    },

    /// Sync skills from node_modules (experimental)
    #[command(name = "experimental_sync")]
    ExperimentalSync,

    /// Install skills from project lock file (experimental)
    #[command(name = "experimental_install")]
    ExperimentalInstall,
}
