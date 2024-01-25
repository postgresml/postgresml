use std::{
    borrow::Cow,
    env::var,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: Config = Config::new();
}

struct Config {
    cms_dir: PathBuf,
    deployment: String,
    dev_mode: bool,
    database_url: String,
    git_sha: String,
    github_stars: String,
    sentry_dsn: Option<String>,
    signup_url: String,
    standalone_dashboard: bool,
    static_dir: PathBuf,
    search_index_dir: PathBuf,
    render_errors: bool,
    css_extension: String,
    js_extension: String,
    assets_domain: Option<String>,
}

impl Config {
    fn new() -> Config {
        let dev_mode = env_is_set("DEV_MODE");

        let signup_url = if dev_mode {
            "/signup"
        } else {
            "https://postgresml.org/signup"
        }
        .to_string();

        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");

        let github_stars = match var("GITHUB_STARS") {
            Ok(stars) => match stars.parse::<f32>() {
                Ok(stars) => format!("{:.1}K", (stars / 1000.0)),
                _ => "1.0K".to_string(),
            },
            _ => "2.0K".to_string(),
        };

        let css_version = env!("CSS_VERSION");
        let js_version = env!("JS_VERSION");

        let css_extension = if dev_mode {
            "css".to_string()
        } else {
            format!("{css_version}.css")
        };
        let js_extension = if dev_mode {
            "js".to_string()
        } else {
            format!("{js_version}.js")
        };

        Config {
            dev_mode,
            database_url: env_string_default("DATABASE_URL", "postgres:///pgml"),
            git_sha: env!("GIT_SHA").to_string(),
            sentry_dsn: env_string_optional("SENTRY_DSN"),
            static_dir: env_path_default("DASHBOARD_STATIC_DIRECTORY", "static"),
            cms_dir: env_path_default("DASHBOARD_CMS_DIRECTORY", "../pgml-cms"),
            search_index_dir: env_path_default("SEARCH_INDEX_DIRECTORY", "search_index"),
            render_errors: env_is_set("RENDER_ERRORS") || dev_mode,
            deployment: env_string_default("DEPLOYMENT", "localhost"),
            signup_url,
            standalone_dashboard: !cargo_manifest_dir.contains("deps") && !cargo_manifest_dir.contains("cloud2"),
            github_stars,
            css_extension,
            js_extension,
            assets_domain: env_string_optional("ASSETS_DOMAIN"),
        }
    }
}

pub fn dev_mode() -> bool {
    CONFIG.dev_mode
}

pub fn database_url<'a>() -> &'a str {
    &CONFIG.database_url
}

pub fn git_sha<'a>() -> &'a String {
    &CONFIG.git_sha
}

pub fn sentry_dsn<'a>() -> &'a Option<String> {
    &CONFIG.sentry_dsn
}
pub fn static_dir<'a>() -> &'a Path {
    &CONFIG.static_dir
}

pub fn cms_dir<'a>() -> &'a Path {
    &CONFIG.cms_dir
}
pub fn search_index_dir<'a>() -> &'a Path {
    &CONFIG.search_index_dir
}
pub fn render_errors() -> bool {
    CONFIG.render_errors
}

pub fn deployment<'a>() -> &'a str {
    &CONFIG.deployment
}
pub fn signup_url<'a>() -> &'a str {
    &CONFIG.signup_url
}
pub fn standalone_dashboard() -> bool {
    CONFIG.standalone_dashboard
}

pub fn github_stars<'a>() -> &'a str {
    &CONFIG.github_stars
}

pub fn css_url(name: &str) -> String {
    let path = PathBuf::from(format!("/dashboard/static/css/{name}"));
    let path = path.with_extension(&CONFIG.css_extension);
    asset_url(path.to_string_lossy())
}

pub fn js_url(name: &str) -> String {
    let path = PathBuf::from(format!("/dashboard/static/js/{name}"));
    let path = path.with_extension(&CONFIG.js_extension);
    asset_url(path.to_string_lossy())
}

pub fn asset_url(path: Cow<str>) -> String {
    match &CONFIG.assets_domain {
        Some(domain) => format!("https://{domain}{path}"),
        None => path.to_string(),
    }
}

pub fn site_domain() -> String {
    String::from("https://postgresml.org")
}

fn env_is_set(name: &str) -> bool {
    var(name).is_ok()
}

fn env_string_default(name: &str, default: &str) -> String {
    match var(name) {
        Ok(value) => value,
        Err(_) => default.to_string(),
    }
}

fn env_string_optional(name: &str) -> Option<String> {
    match var(name) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

fn env_path_default(name: &str, default: &str) -> PathBuf {
    match var(name) {
        Ok(value) => PathBuf::from(value),
        Err(_) => PathBuf::from(default),
    }
}
