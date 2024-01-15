use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use url::Url;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RemoteKey {
    pub scheme: String,
    pub username: String,
    pub host: String,
    pub port_opt: Option<u16>,
}

pub struct RemoteCache {
    pub cache: HashMap<RemoteKey, Arc<Mutex<dyn remotefs::RemoteFs>>>,
}

impl RemoteCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, url: &Url) -> Result<Arc<Mutex<dyn remotefs::RemoteFs>>, Box<dyn Error>> {
        let scheme = url.scheme();
        match scheme {
            "sftp" => match url.host_str() {
                Some(host) => {
                    //TODO: percent decode url data!
                    let mut username = url.username().to_string();
                    if username.is_empty() {
                        // If username is empty, try username
                        username = whoami::username();
                    }

                    let key = RemoteKey {
                        scheme: scheme.to_string(),
                        username,
                        host: host.to_string(),
                        port_opt: url.port(),
                    };

                    //TODO: remote fs if not used or has failed
                    let fs_mutex = self
                        .cache
                        .entry(key)
                        .or_insert_with_key(|key| {
                            let mut opts = remotefs_ssh::SshOpts::new(&key.host);
                            if let Some(port) = key.port_opt {
                                opts = opts.port(port);
                            }

                            opts = opts.username(&key.username);
                            if let Some(password) = url.password() {
                                opts = opts.password(password);
                            }

                            if let Some(home_dir) = dirs::home_dir() {
                                let ssh_dir = home_dir.join(".ssh");

                                // Add default ssh config if it exists
                                let config_file = ssh_dir.join("config");
                                if config_file.exists() {
                                    opts = opts.config_file(
                                        &config_file,
                                        remotefs_ssh::SshConfigParseRule::STRICT,
                                    );
                                }

                                // Add id_rsa key auth if no password supplied and it exists
                                if url.password().is_none() {
                                    let id_rsa_file = ssh_dir.join("id_rsa");
                                    if id_rsa_file.exists() {
                                        opts = opts.key_storage(Box::new(SimpleSshKeyStorage {
                                            path: id_rsa_file,
                                        }));
                                    }
                                }
                            }

                            let fs = remotefs_ssh::SftpFs::new(opts);
                            Arc::new(Mutex::new(fs))
                        })
                        .clone();
                    {
                        let mut fs = fs_mutex.lock().unwrap();
                        if !fs.is_connected() {
                            fs.connect()?;
                        }
                        //TODO: what to do with empty path?
                        let url_path = Path::new(url.path());
                        if fs.pwd()? != url_path {
                            fs.change_dir(url_path)?;
                        }
                    }
                    Ok(fs_mutex)
                }
                None => Err(format!("failed to connect to {:?}: no host", url).into()),
            },
            _ => Err(format!("failed to connect to {:?}: unsupported scheme", url).into()),
        }
    }
}

struct SimpleSshKeyStorage {
    path: PathBuf,
}

impl remotefs_ssh::SshKeyStorage for SimpleSshKeyStorage {
    fn resolve(&self, _host: &str, _username: &str) -> Option<PathBuf> {
        Some(self.path.clone())
    }
}
