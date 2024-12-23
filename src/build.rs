use std::process::Command;

use git2::Repository;
use serde::Deserialize;

#[derive(Deserialize)]
struct Cargo {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

#[derive(Deserialize)]
struct Release {
    build: Build,
}

#[derive(Deserialize)]
struct Build {
    platforms: Vec<Platform>,
}

#[derive(Deserialize)]
struct Platform {
    target: String,
    platform: String,
    arch: String,
}

pub fn build(tag: String) {
    Command::new("git")
        .args(["fetch", "--tags", "origin"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let release_toml = std::fs::read_to_string("release.toml").unwrap();
    let release_toml: Release = toml::from_str(&release_toml).unwrap();
    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let cargo_toml: Cargo = toml::from_str(&cargo_toml).unwrap();

    let repo = Repository::open(".").unwrap();
    let branch = repo.head().unwrap();
    let branch = branch.shorthand().unwrap();

    let (object, reference) = repo.revparse_ext(&tag).unwrap();
    repo.checkout_tree(&object, None).unwrap();
    repo.set_head(reference.unwrap().name().unwrap()).unwrap();

    std::fs::create_dir("build").unwrap_or(());

    for platform in release_toml.build.platforms {
        let target = platform.target;

        Command::new("cargo")
            .args(["build", "--release", "--target", target.as_str()])
            .envs([("RELEASE_VERSION", tag.as_str())])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        let out_path = format!(
            "build/{}-{}-{}-{}{}",
            cargo_toml.package.name,
            tag,
            platform.platform,
            platform.arch,
            if platform.platform == "windows" {
                ".exe"
            } else {
                ""
            }
        );
        std::fs::rename(
            format!(
                "target/{}/release/{}{}",
                target,
                cargo_toml.package.name,
                if platform.platform == "windows" {
                    ".exe"
                } else {
                    ""
                }
            ),
            &out_path,
        )
        .unwrap();

        let ver = semver::Version::parse(&tag[1..]).unwrap();

        Command::new("gh")
            .args(["release", "upload", &tag, &out_path, "--clobber"])
            .envs([
                ("RELEASE_VERSION", tag.as_str()),
                (
                    "RELEASE_CHANNEL",
                    ver.pre.split('.').collect::<Vec<&str>>()[0],
                ),
                ("RELEASE_MAJOR", ver.major.to_string().as_str()),
                ("RELEASE_MINOR", ver.minor.to_string().as_str()),
                ("RELEASE_PATCH", ver.patch.to_string().as_str()),
            ])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        std::fs::remove_file(out_path).unwrap();
    }

    let (object, reference) = repo.revparse_ext(branch).unwrap();
    repo.checkout_tree(&object, None).unwrap();
    repo.set_head(reference.unwrap().name().unwrap()).unwrap();
}
