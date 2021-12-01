use crate::config_file::JuliaupConfigChannel;
use crate::operations::{install_version, create_symlink};
use crate::jsonstructs_versionsdb::JuliaupVersionDB;
use crate::config_file::JuliaupConfig;
use crate::operations::garbage_collect_versions;
use crate::config_file::{load_config_db, save_config_db};
use crate::versions_file::load_versions_db;
use anyhow::{Context, Result,anyhow,bail};

fn update_channel(config_db: &mut JuliaupConfig, channel: &String, version_db: &JuliaupVersionDB, ignore_linked_channel: bool) -> Result<()> {    
    let current_version = 
        config_db.installed_channels.get(channel).ok_or(anyhow!("asdf"))?;

    match current_version {
        JuliaupConfigChannel::SystemChannel {version} => {
            let should_version = version_db.available_channels.get(channel).ok_or(anyhow!("asdf"))?;

            if &should_version.version != version {
                install_version(&should_version.version, config_db, version_db)
                    .with_context(|| format!("Failed to install '{}' while updating channel '{}'.", should_version.version, channel))?;

                config_db.installed_channels.insert(
                    channel.clone(),
                    JuliaupConfigChannel::SystemChannel {
                        version: should_version.version.clone(),
                    },
                );

                if std::env::consts::OS != "windows" && config_db.create_symlinks {
                    create_symlink(
                        &JuliaupConfigChannel::SystemChannel {
                            version: should_version.version.clone(),
                        },
                        &format!("julia-{}", channel),
                    )?;
                }
            }
        },
        JuliaupConfigChannel::LinkedChannel {command: _, args: _} => if !ignore_linked_channel {
            bail!("Failed to update '{}' because it is a linked channel.", channel)
        } else {()}
    }

    Ok(())
}

pub fn run_command_update(channel: Option<String>) -> Result<()> {
    let version_db =
        load_versions_db().with_context(|| "`update` command failed to load versions db.")?;

    let mut config_data = load_config_db()
        .with_context(|| "`update` command failed to load configuration file.")?;

    match channel {
        None => {
            for (k,_) in config_data.installed_channels.clone() {
                update_channel(&mut config_data, &k, &version_db, true)?;
            }

        },
        Some(channel) => {
            if !config_data.installed_channels.contains_key(&channel) {
                bail!("'{}' cannot be updated because it is currently not installed.", channel);
            }

            update_channel(&mut config_data, &channel, &version_db, false)?;
        }
    };

    garbage_collect_versions(&mut config_data)?;

    save_config_db(&config_data)
        .with_context(|| "`update` command failed to save configuration db.")?;

    Ok(())
}
