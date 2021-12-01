use crate::operations::{install_version, create_symlink};
use crate::config_file::JuliaupConfigChannel;
use crate::config_file::{load_config_db, save_config_db};
use crate::versions_file::load_versions_db;
use anyhow::{anyhow, bail, Context, Result};

pub fn run_command_add(channel: String) -> Result<()> {
    let version_db =
        load_versions_db().with_context(|| "`add` command failed to load versions db.")?;

    let required_version = &version_db
        .available_channels
        .get(&channel)
        .ok_or(anyhow!(
            "'{}' is not a valid Julia version or channel name.",
            &channel
        ))?
        .version;

    let mut config_data =
        load_config_db().with_context(|| "`add` command failed to load configuration file.")?;

    if config_data.installed_channels.contains_key(&channel) {
        bail!("'{}' is already installed.", &channel);
    }
    
    install_version(&required_version, &mut config_data, &version_db)?;

    config_data.installed_channels.insert(
        channel.clone(),
        JuliaupConfigChannel::SystemChannel {
            version: required_version.clone(),
        },
    );

    if config_data.default.is_none() {
        config_data.default = Some(channel.clone());
    }

    save_config_db(&config_data)
        .with_context(|| format!("Failed to save configuration file from `add` command after '{}' was installed.", channel))?;

    if std::env::consts::OS != "windows" && config_data.create_symlinks {
        create_symlink(
            &JuliaupConfigChannel::SystemChannel {
                version: required_version.clone(),
            },
            &format!("julia-{}", channel),
        )?;
    }

    Ok(())
}
