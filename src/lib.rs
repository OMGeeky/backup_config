use prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
pub mod prelude {
    pub use super::Conf;
    pub use confique::Config;
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Config)]
pub struct Conf {
    /// The Database Connection URL
    #[config(default = "sqlite://data.db?mode=rwc")]
    pub db_url: String,
    /// The folder where all the logs will be stored.
    #[config(default = "/var/tmp/twba/logs/")]
    pub log_folder: String,
    /// How many items should be processed in a single run.
    ///
    /// After this number is reached the program will stop and wait for the next run.
    #[config(default = 5)]
    pub max_items_to_process: u64,

    #[config(nested)]
    pub notifier: Notifier,
    #[config(nested)]
    pub twitch: Twitch,
    #[config(nested)]
    pub google: Google,

    /// The path to the folder where the videos will be downloaded to.
    #[config(default = "/var/tmp/twba/videos/")]
    pub download_folder_path: String,

    /// How many videos should be downloaded at the same time.
    ///
    /// If this number is reached because the uploader can't keep up, the downloader will wait until
    /// the number of downloads is below this number again.
    ///
    /// Set to 0 or leave empty to disable the limit.
    pub maximum_downloaded_videos: Option<u64>,
}

impl Conf {
    pub fn log_path(&self) -> PathBuf {
        #[cfg(feature = "home")]
        let path = &shellexpand::tilde(&self.log_folder).into_owned();

        #[cfg(not(feature = "home"))]
        let path = &self.log_folder;
        PathBuf::from(path)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Config)]
pub struct Notifier {
    /// The URL for the webhook
    pub webhook_url: Option<String>,
    pub smtp: Option<Smtp>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Config)]
pub struct Smtp {
    /// The SMTP server address
    pub server: String,
    /// The SMTP server port
    pub port: u16,
    /// The SMTP username
    pub username: String,
    /// The SMTP password
    pub password: String,
    /// The email address to send the email from
    pub from: String,
    /// The email address to send the email to
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Config)]
pub struct Google {
    /// The path for the auth code file.
    ///
    /// This file will contain the response of the auth request.
    ///
    /// The file should be deleted after the auth response is used.
    #[config(default = "tmp/twba/auth/code.txt")]
    pub path_auth_code: String,
    /// The path for the auth cache file for each user. '{user}' will be replaced with the user's login.
    #[config(default = "tmp/twba/auth/{user}.txt")]
    pub path_auth_cache: String,

    /// Decides if the auth should go to the localhost or the public IP/Server
    #[config(default = false)]
    pub local_auth_redirect: bool,

    /// Decides if the auth-code should be stored in a file or be read by stdin
    #[config(default = true)]
    pub use_file_auth_response: bool,

    /// The frequency for reading the auth file in seconds
    #[config(default = 10)]
    pub auth_file_read_frequency: u64,
    /// The timeout for reading the auth file in seconds
    #[config(default = 86400)] // 24 hours
    pub auth_file_read_timeout: u64,

    /// The project ID for the Google Cloud project
    #[config(default = "twitchbackup-1")]
    pub project_id: String,

    #[config(nested)]
    pub youtube: Youtube,

    #[config(nested)]
    pub bigquery: BigQuery,
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Config)]
pub struct BigQuery {
    /// The path to the service account file for the BigQuery API
    #[config(default = "auth/bigquery_service_account.json")]
    pub service_account_path: String,

    /// The dataset ID for the BigQuery dataset
    #[config(default = "backup_data")]
    pub dataset_id: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Config)]
pub struct Youtube {
    /// The path to the client secret file for the youtube API
    #[config(default = "auth/youtube_client_secret.json")]
    pub client_secret_path: String,
    /// The default tags to use for youtube videos.
    ///
    /// can be overridden per user
    #[config(default = [])]
    pub default_tags: Vec<String>,
    /// The default description template to use for youtube videos.
    ///
    /// can be overridden per user
    ///
    #[config(default = "test description for \"$$video_title$$\"")]
    pub default_description_template: String,
    /// The default for the targeted duration (in minutes) a youtube video should be.
    ///
    /// If the video is longer than the hard cap it will be split so
    /// each part except for the last will have the length of the soft cap.    
    ///
    /// can be overridden per user
    #[config(default = 300)]
    pub default_video_length_minutes_soft_cap: i64,
    /// The default for the maximum duration (in minutes) a youtube video should be.
    ///
    /// If the video is longer than this, it will be split into multiple
    /// videos (each besides the last the length of the soft cap).
    ///
    /// can be overridden per user
    #[config(default = 359)]
    pub default_video_length_minutes_hard_cap: i64,
}
#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Config)]
pub struct Twitch {
    /// The client ID for the Twitch API
    pub client_id: String,

    /// The client secret for the Twitch API
    pub client_secret: String,

    /// This is the Client-ID used for downloading videos.
    #[config(default = "kimne78kx3ncx6brgo4mv6wki5h1ko")]
    pub downloader_id: String,

    /// The number of threads to use for downloading videos.
    #[config(default = 50)]
    pub downloader_thread_count: u64,
}

pub fn get_default_builder() -> confique::Builder<Conf> {
    #[cfg(feature = "tracing")]
    tracing::info!("building default config");
    let conf = Conf::builder();
    #[cfg(feature = "env")]
    let conf = conf.env().file(
        std::env::var("TWBA_CONFIG")
            .map(|v| {
                #[cfg(feature = "tracing")]
                tracing::info!("using '{}' as primary config source after env", v);
                v
            })
            .unwrap_or_else(|_| {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not get config location from env");
                "./settings.toml".to_string()
            }),
    );
    #[cfg(feature = "home")]
    let conf = conf.file(shellexpand::tilde("~/twba/config.toml").into_owned());
    conf
}
