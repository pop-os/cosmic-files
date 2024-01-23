use remotefs::{RemoteError, RemoteFs};
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
    pub cache: HashMap<RemoteKey, Result<Arc<Mutex<dyn RemoteFs>>, RemoteError>>,
}

impl RemoteCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    #[cfg(not(feature = "remotefs-ssh"))]
    fn get_sftp(
        &mut self,
        _url: &Url,
    ) -> Result<(Arc<Mutex<dyn RemoteFs>>, PathBuf), Box<dyn Error>> {
        Err(format!("remotefs-ssh feature not enabled").into())
    }

    #[cfg(feature = "remotefs-ssh")]
    fn get_sftp(
        &mut self,
        url: &Url,
    ) -> Result<(Arc<Mutex<dyn RemoteFs>>, PathBuf), Box<dyn Error>> {
        struct SimpleSshKeyStorage {
            path: PathBuf,
        }

        impl remotefs_ssh::SshKeyStorage for SimpleSshKeyStorage {
            fn resolve(&self, _host: &str, _username: &str) -> Option<PathBuf> {
                Some(self.path.clone())
            }
        }

        let key = match url.host_str() {
            Some(host) => {
                //TODO: percent decode url data!
                let mut username = url.username().to_string();
                if username.is_empty() {
                    // If username is empty, try username
                    username = whoami::username();
                }

                RemoteKey {
                    scheme: url.scheme().to_string(),
                    username,
                    host: host.to_string(),
                    port_opt: url.port(),
                }
            }
            None => return Err(format!("failed to connect to {}: no host", url).into()),
        };

        let fs_mutex_res = self
            .cache
            .entry(key.clone())
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
                        opts = opts
                            .config_file(&config_file, remotefs_ssh::SshConfigParseRule::STRICT);
                    }

                    // Add id_rsa key auth if no password supplied and it exists
                    if url.password().is_none() {
                        let id_rsa_file = ssh_dir.join("id_rsa");
                        if id_rsa_file.exists() {
                            opts = opts
                                .key_storage(Box::new(SimpleSshKeyStorage { path: id_rsa_file }));
                        }
                    }
                }

                let fs = remotefs_ssh::SftpFs::new(opts);
                Ok(Arc::new(Mutex::new(fs)))
            })
            .clone();

        match fs_mutex_res {
            Ok(fs_mutex) => {
                {
                    let mut fs = fs_mutex.lock().unwrap();
                    if !fs.is_connected() {
                        fs.connect()?;
                    }
                }
                Ok((fs_mutex, PathBuf::from(url.path())))
            }
            Err(err) => {
                // Remove failed connection
                self.cache.remove(&key);
                Err(err.into())
            }
        }
    }

    #[cfg(not(feature = "remotefs-smb"))]
    fn get_smb(
        &mut self,
        _url: &Url,
    ) -> Result<(Arc<Mutex<dyn RemoteFs>>, PathBuf), Box<dyn Error>> {
        Err(format!("remotefs-smb feature not enabled").into())
    }

    #[cfg(feature = "remotefs-smb")]
    fn get_smb(
        &mut self,
        url: &Url,
    ) -> Result<(Arc<Mutex<dyn RemoteFs>>, PathBuf), Box<dyn Error>> {
        let key = match url.host_str() {
            Some(host) => {
                //TODO: percent decode url data!
                let mut username = url.username().to_string();
                if username.is_empty() {
                    // If username is empty, try username
                    username = whoami::username();
                }

                RemoteKey {
                    scheme: url.scheme().to_string(),
                    username,
                    host: host.to_string(),
                    port_opt: url.port(),
                }
            }
            None => return Err(format!("failed to connect to {}: no host", url).into()),
        };

        let fs_mutex_res = self
            .cache
            .entry(key.clone())
            .or_insert_with_key(|key| {
                let mut creds = remotefs_smb::SmbCredentials::default();

                let mut server = format!("smb://{}", key.host);
                if let Some(port) = key.port_opt {
                    server.push_str(&format!(":{}", port));
                }
                creds = creds.server(server);

                //TODO: allow selecting workgroup
                creds = creds.workgroup("WORKGROUP");
                creds = creds.username(&key.username);
                if let Some(password) = url.password() {
                    creds = creds.password(password);
                }

                let opts = remotefs_smb::SmbOptions::default()
                    .case_sensitive(false)
                    .one_share_per_server(true);
                let fs = remotefs_smb::SmbFs::try_new(creds, opts)?;
                Ok(Arc::new(Mutex::new(fs)))
            })
            .clone();

        match fs_mutex_res {
            Ok(fs_mutex) => {
                {
                    let mut fs = fs_mutex.lock().unwrap();
                    if !fs.is_connected() {
                        fs.connect()?;
                    }
                }
                Ok((fs_mutex, PathBuf::from(url.path())))
            }
            Err(err) => {
                // Remove failed connection
                self.cache.remove(&key);
                Err(err.into())
            }
        }
    }

    pub fn get(
        &mut self,
        url: &Url,
    ) -> Result<(Arc<Mutex<dyn RemoteFs>>, PathBuf), Box<dyn Error>> {
        //TODO: strip password from URL, return URL after connect?
        match url.scheme() {
            "sftp" => self.get_sftp(url),
            "smb" => self.get_smb(url),
            _ => Err(format!("failed to connect to {}: unsupported scheme", url).into()),
        }
    }
}
