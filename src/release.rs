use git2::Repository;
use semver::Version;
use serde::Deserialize;
use std::{
    fmt::{self, Display},
    process::Command,
};

#[derive(Deserialize)]
struct CargoToml {
    package: CargoTomlPkg,
}

#[derive(Deserialize)]
struct CargoTomlPkg {
    version: String,
}

#[derive(Debug)]
enum ReleaseType {
    Alpha(u64),
    Beta(u64),
    Candidate(u64),
    Stable,
}

impl Display for ReleaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseType::Alpha(_) => write!(f, "alpha"),
            ReleaseType::Beta(_) => write!(f, "beta"),
            ReleaseType::Candidate(_) => write!(f, "rc"),
            ReleaseType::Stable => write!(f, "stable"),
        }
    }
}

fn get_releaseinfo() -> (u64, u64, u64, ReleaseType) {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let cargo_toml: CargoToml = toml::from_str(&cargo_toml).unwrap();
    let version = cargo_toml.package.version;
    let version = Version::parse(&version).unwrap();
    (
        version.major,
        version.minor,
        version.patch,
        match version.pre.as_str() {
            s if s.starts_with("alpha") => ReleaseType::Alpha(if s.len() > 6 {
                s[6..].parse().unwrap_or(1)
            } else {
                1
            }),
            s if s.starts_with("beta") => ReleaseType::Beta(if s.len() > 5 {
                s[5..].parse().unwrap_or(1)
            } else {
                1
            }),
            s if s.starts_with("rc") => ReleaseType::Candidate(if s.len() > 3 {
                s[3..].parse().unwrap_or(1)
            } else {
                1
            }),
            _ => ReleaseType::Stable,
        },
    )
}

fn force_merge(repo: &Repository, source_branch: &str, target_branch: &str) {
    let source_ref = repo
        .find_reference(&format!("refs/heads/{}", source_branch))
        .unwrap();
    let source_commit = source_ref.peel_to_commit().unwrap();

    // Hole oder erstelle den Ziel-Branch (beta)
    let mut target_ref = match repo.find_reference(&format!("refs/heads/{}", target_branch)) {
        Ok(r) => r,
        Err(_) => repo
            .branch(target_branch, &source_commit, true)
            .unwrap()
            .into_reference(),
    };

    // Setze den Ziel-Branch auf den Commit des Quell-Branches
    target_ref
        .set_target(source_commit.id(), "Force merge by overwriting")
        .unwrap();

    // Aktualisiere die HEAD-Referenz auf den Ziel-Branch (optional, falls `beta` ausgecheckt werden soll)
    repo.set_head(&format!("refs/heads/{}", target_branch))
        .unwrap();
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .unwrap();
}

pub fn release(r#type: String) {
    let (major, minor, patch, release_type) = get_releaseinfo();
    let mut new_major = major;
    let mut new_minor = minor;
    let mut new_patch = patch;
    let mut new_channel = release_type;
    let mut new_channelver = match new_channel {
        ReleaseType::Alpha(n) => n,
        ReleaseType::Beta(n) => n,
        ReleaseType::Candidate(n) => n,
        ReleaseType::Stable => 0,
    };

    match r#type.as_str() {
        "major" => {
            new_major += 1;
            new_minor = 0;
            new_patch = 0;
            new_channel = ReleaseType::Stable;
        }
        "minor" => {
            new_minor += 1;
            new_patch = 0;
            new_channel = ReleaseType::Stable;
        }
        "patch" => {
            new_patch += 1;
            new_channel = ReleaseType::Stable;
        }
        "alpha-major" => match new_channel {
            ReleaseType::Stable => {
                new_major += 1;
                new_minor = 0;
                new_patch = 0;
                new_channelver = 1;
                new_channel = ReleaseType::Alpha(1);
            }
            _ => {
                eprintln!("You can't change the major version in a non-stable release");
                std::process::exit(1);
            }
        },
        "alpha" => match new_channel {
            ReleaseType::Stable => {
                new_minor += 1;
                new_patch = 0;
                new_channelver = 1;
                new_channel = ReleaseType::Alpha(1);
            }
            ReleaseType::Alpha(_) => {
                new_channelver += 1;
                new_channel = ReleaseType::Alpha(new_channelver);
            }
            _ => {
                eprintln!("You can't downgrade to alpha from beta or rc");
            }
        },
        "beta-major" => match new_channel {
            ReleaseType::Stable => {
                new_major += 1;
                new_minor = 0;
                new_patch = 0;
                new_channelver = 1;
                new_channel = ReleaseType::Beta(1);
            }
            _ => {
                eprintln!("You can't change the major version in a non-stable release");
                std::process::exit(1);
            }
        },
        "beta" => match new_channel {
            ReleaseType::Stable => {
                new_minor += 1;
                new_patch = 0;
                new_channelver = 1;
                new_channel = ReleaseType::Beta(1);
            }
            ReleaseType::Beta(_) => {
                new_channelver += 1;
                new_channel = ReleaseType::Beta(new_channelver);
            }
            ReleaseType::Alpha(_) => {
                new_channelver = 1;
                new_channel = ReleaseType::Beta(new_channelver);
            }
            _ => {
                eprintln!("You can't downgrade to beta from rc");
            }
        },
        "rc-major" => match new_channel {
            ReleaseType::Stable => {
                new_major += 1;
                new_minor = 0;
                new_patch = 0;
                new_channelver = 1;
                new_channel = ReleaseType::Candidate(1);
            }
            _ => {
                eprintln!("You can't change the major version in a non-stable release");
                std::process::exit(1);
            }
        },
        "rc" => match new_channel {
            ReleaseType::Stable => {
                new_minor += 1;
                new_patch = 0;
                new_channelver = 1;
                new_channel = ReleaseType::Candidate(1);
            }
            ReleaseType::Candidate(_) => {
                new_channelver += 1;
                new_channel = ReleaseType::Candidate(new_channelver);
            }
            _ => {
                new_channelver = 1;
                new_channel = ReleaseType::Candidate(new_channelver);
            }
        },
        _ => {
            eprintln!("Invalid release type");
            std::process::exit(1);
        }
    }

    let ver = match new_channel {
        ReleaseType::Stable => format!("{}.{}.{}", new_major, new_minor, new_patch),
        _ => {
            if new_channelver == 1 {
                format!("{}.{}.{}-{}", new_major, new_minor, new_patch, new_channel)
            } else {
                format!(
                    "{}.{}.{}-{}.{}",
                    new_major, new_minor, new_patch, new_channel, new_channelver
                )
            }
        }
    };

    println!("Releasing new version: {}", ver);

    let repo = Repository::open(".").unwrap();

    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let mut doc = cargo_toml.parse::<toml_edit::DocumentMut>().unwrap();
    doc["package"]["version"] = toml_edit::value(ver.clone());
    std::fs::write("Cargo.toml", doc.to_string()).unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("Cargo.toml")).unwrap();
    let oid = index.write_tree().unwrap();

    repo.commit(
        Some("HEAD"),
        &repo.signature().unwrap(),
        &repo.signature().unwrap(),
        &format!("release: v{}", ver),
        &repo.find_tree(oid).unwrap(),
        &[&repo.head().unwrap().peel_to_commit().unwrap()],
    )
    .unwrap();

    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    repo.reset(commit.as_object(), git2::ResetType::Mixed, None)
        .unwrap();

    match new_channel {
        ReleaseType::Alpha(_) => {
            force_merge(&repo, repo.head().unwrap().shorthand().unwrap(), "alpha");
        }
        ReleaseType::Beta(_) => {
            force_merge(&repo, repo.head().unwrap().shorthand().unwrap(), "alpha");
            force_merge(&repo, "alpha", "beta");
        }
        ReleaseType::Candidate(_) => {
            force_merge(&repo, repo.head().unwrap().shorthand().unwrap(), "alpha");
            force_merge(&repo, "alpha", "beta");
            force_merge(&repo, "beta", "rc");
        }
        ReleaseType::Stable => {
            force_merge(&repo, repo.head().unwrap().shorthand().unwrap(), "alpha");
            force_merge(&repo, "alpha", "beta");
            force_merge(&repo, "beta", "rc");
            force_merge(&repo, "rc", "main");
        }
    }

    Command::new("git push").spawn().unwrap().wait().unwrap();
}
