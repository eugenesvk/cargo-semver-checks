#![allow(unused_imports,unused_variables,unreachable_code,dead_code,non_upper_case_globals)]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use cargo_semver_checks::{
    GlobalConfig, PackageSelection, ReleaseType, Rustdoc, ScopeSelection, SemverQuery,
};
use clap::{Args, Parser, Subcommand};

fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();

    let Cargo::SemverChecks(args) = Cargo::parse();
    // println!("×××× main");
    // todo: find out what calls generate_rustdoc that generates pub_module_level_const_missing.json
    // and then what queries the file
    println!("args.command {:?}",args.command);
    println!("args.check_release {:?}",args.check_release);
    let check: cargo_semver_checks::Check = args.check_release.into();
    // etag7: calls @lib.rs check_release that calls generate_versioned_crates to call @rustdoc_gen to generate rustdoc actual file pub_module_level_const_missing
    let report = check.check_release()?;

    // CheckRelease { manifest: Manifest { manifest_path: None }
    // , workspace: Workspace { package: [], workspace: false, all: false, exclude: [] }
    // , current_rustdoc: None, baseline_version: None, baseline_rev: None, baseline_rustdoc: None, release_type: None, default_features: false
    // , only_explicit_features: false, features: [], baseline_features: [], current_features: [], all_features: false, verbosity: Verbosity { verbose: 0 , quiet: 0
    // , baseline_root: Some("pub_module_level_const_missing\\old")
    // , phantom: PhantomData<clap_verbosity_flag::InfoLevel> } }

    if report.success() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(version, propagate_version = true)]
enum Cargo {
    SemverChecks(SemverChecks),
}

#[derive(Debug, Args)] #[command(args_conflicts_with_subcommands = true)]
struct SemverChecks {
  #[clap(flatten)]      	check_release	: CheckRelease,
  #[command(subcommand)]	command      	: Option<SemverChecksCommands>,
}

/// Check your crate for semver violations.
#[derive(Debug, Subcommand)]
enum SemverChecksCommands {
    #[command(alias = "diff-files")]
    CheckRelease(CheckRelease),
}

#[derive(Debug, Args)]
struct CheckRelease {
    #[command(flatten, next_help_heading = "Current")] pub manifest: clap_cargo::Manifest,
    #[command(flatten, next_help_heading = "Current")] pub workspace: clap_cargo::Workspace,
    /// The current rustdoc json output to test for semver violations.
    #[arg(
        long,
        short_alias = 'c',
        alias = "current",
        value_name = "JSON_PATH",
        help_heading = "Current",
        requires = "baseline_rustdoc",
        conflicts_with_all = [
            "default_features",
            "only_explicit_features",
            "features",
            "baseline_features",
            "current_features",
            "all_features",
        ]
    )]
    current_rustdoc: Option<PathBuf>,

    /// Version from registry to lookup for a baseline
    #[arg(
        long,
        value_name = "X.Y.Z",
        help_heading = "Baseline",
        group = "baseline"
    )]
    baseline_version: Option<String>,

    /// Git revision to lookup for a baseline
    #[arg(
        long,
        value_name = "REV",
        help_heading = "Baseline",
        group = "baseline"
    )]
    baseline_rev: Option<String>,

    /// Directory containing baseline crate source
    #[arg(
        long,
        value_name = "MANIFEST_ROOT",
        help_heading = "Baseline",
        group = "baseline"
    )]
    baseline_root: Option<PathBuf>,

    /// The rustdoc json file to use as a semver baseline.
    #[arg(
        long,
        short_alias = 'b',
        alias = "baseline",
        value_name = "JSON_PATH",
        help_heading = "Baseline",
        group = "baseline",
        conflicts_with_all = [
            "default_features",
            "only_explicit_features",
            "features",
            "baseline_features",
            "current_features",
            "all_features",
        ]
    )]
    baseline_rustdoc: Option<PathBuf>,

    /// Sets the release type instead of deriving it from the version number.
    #[arg(
        value_enum,
        long,
        value_name = "TYPE",
        help_heading = "Overrides",
        group = "overrides"
    )]
    release_type: Option<ReleaseType>,

    /// Use only the crate-defined default features, as well as any features
    /// added explicitly via other flags.
    ///
    /// Using this flag disables the heuristic that enables all features
    /// except `unstable`, `nightly`, `bench`, `no_std`, and ones starting with prefixes
    /// `_`, `unstable_`, `unstable-`.
    #[arg(
        long,
        help_heading = "Features",
        conflicts_with = "only_explicit_features"
    )]
    default_features: bool,

    /// Use no features except ones explicitly added by other flags.
    ///
    /// Using this flag disables the heuristic that enables all features
    /// except `unstable`, `nightly`, `bench`, `no_std`, and ones starting with prefixes
    /// `_`, `unstable_`, `unstable-`.
    #[arg(long, help_heading = "Features")]
    only_explicit_features: bool,

    /// Add a feature to the set of features being checked.
    /// The feature will be used in both the baseline and the current version
    /// of the crate.
    #[arg(long, value_name = "NAME", help_heading = "Features")]
    features: Vec<String>,

    /// Add a feature to the set of features being checked.
    /// The feature will be used in the baseline version of the crate only.
    #[arg(long, value_name = "NAME", help_heading = "Features")]
    baseline_features: Vec<String>,

    /// Add a feature to the set of features being checked.
    /// The feature will be used in the current version of the crate only.
    #[arg(long, value_name = "NAME", help_heading = "Features")]
    current_features: Vec<String>,

    /// Use all the features, including features named
    /// `unstable`, `nightly`, `bench`, `no_std` or starting with prefixes
    /// `_`, `unstable_`, `unstable-` that are otherwise disabled by default.
    #[arg(
        long,
        help_heading = "Features",
        conflicts_with_all = [
            "default_features",
            "only_explicit_features",
            "features",
            "baseline_features",
            "current_features",
        ]
    )]
    all_features: bool,

    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
}

impl From<CheckRelease> for cargo_semver_checks::Check {
    fn from(value: CheckRelease) -> Self {
        let project_root = std::env::current_dir().expect("can't determine current directory");
        let (current, current_project_root) = (Rustdoc::from_root(&project_root), Some(project_root));
        let mut check = Self::new(current);
        // if value.workspace.all || value.workspace.workspace {
        //     println!("✗✗ sdfsasdf");
        //     // Specified explicit `--workspace` or `--all`.
        //     let mut selection = PackageSelection::new(ScopeSelection::Workspace);
        //     selection.with_excluded_packages(value.workspace.exclude);
        //     check.with_package_selection(selection);
        // } else if !value.workspace.package.is_empty() {
        //     println!("✗✗ sdfsasdf else ");
        //     // Specified explicit `--package`.
        //     check.with_packages(value.workspace.package);
        // } else if !value.workspace.exclude.is_empty() {
        //     println!("✗✗ sdfsasdf else 3");
        //     // Specified `--exclude` without `--workspace/--all`.
        //     // Leave the scope selection to the default ("workspace if the manifest is a workspace")
        //     // while excluding any specified packages.
        //     let mut selection = PackageSelection::new(ScopeSelection::DefaultMembers);
        //     selection.with_excluded_packages(value.workspace.exclude);
        //     check.with_package_selection(selection);
        // }
        let custom_baseline = {
            if let Some(baseline_version) = value.baseline_version {
                Some(Rustdoc::from_registry(baseline_version))
            } else if let Some(baseline_rev) = value.baseline_rev {
                let root = if let Some(baseline_root) = value.baseline_root {
                    baseline_root
                } else if let Some(current_root) = current_project_root {
                    current_root
                } else {
                    std::env::current_dir().expect("can't determine current directory")
                };
                Some(Rustdoc::from_git_revision(root, baseline_rev))
            } else if let Some(baseline_rustdoc) = value.baseline_rustdoc {
                Some(Rustdoc::from_path(baseline_rustdoc))
            } else {
                // Either there's a manually-set baseline root path, or fall through
                // to the default behavior.
                value.baseline_root.map(Rustdoc::from_root)
            }
        };
        if let Some(baseline) = custom_baseline {
            check.with_baseline(baseline);
        }
        if let Some(log_level) = value.verbosity.log_level() {
            check.with_log_level(log_level);
        }
        if let Some(release_type) = value.release_type {
            check.with_release_type(release_type);
        }

        if value.all_features {
            check.with_all_features();
        } else if value.default_features {
            check.with_default_features();
        } else if value.only_explicit_features {
            check.with_only_explicit_features();
        } else {
            check.with_heuristically_included_features();
        }
        let mut mutual_features = value.features;
        let mut current_features = value.current_features;
        let mut baseline_features = value.baseline_features;
        current_features.append(&mut mutual_features.clone());
        baseline_features.append(&mut mutual_features);
        check.with_extra_features(current_features, baseline_features);

        check
    }
}
