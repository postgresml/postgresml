use std::env::var;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: Config = Config::new();
}

struct Config {
    blogs_dir: PathBuf,
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
    css_version: String,
    js_version: String,
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

        let github_stars = match var("GITHUB_STARS") {
            Ok(stars) => match stars.parse::<f32>() {
                Ok(stars) => format!("{:.1}K", (stars / 1000.0)),
                _ => "1.0K".to_string(),
            },
            _ => "2.0K".to_string(),
        };

        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");

        Config {
            dev_mode,
            database_url: env_string_default("DATABASE_URL", "postgres:///pgml"),
            git_sha: env_string_required("GIT_SHA"),
            sentry_dsn: env_string_optional("SENTRY_DSN"),
            static_dir: env_path_default("DASHBOARD_STATIC_DIRECTORY", "static"),
            blogs_dir: env_path_default("DASHBOARD_CONTENT_DIRECTORY", "content"),
            cms_dir: env_path_default("DASHBOARD_CMS_DIRECTORY", "../pgml-docs"),
            search_index_dir: env_path_default("SEARCH_INDEX_DIRECTORY", "search_index"),
            render_errors: env_is_set("RENDER_ERRORS") || dev_mode,
            deployment: env_string_default("DEPLOYMENT", "localhost"),
            signup_url,
            standalone_dashboard: !cargo_manifest_dir.contains("deps")
                && !cargo_manifest_dir.contains("cloud2"),
            github_stars: github_stars,
            css_version: env_string_default("CSS_VERSION", "1"),
            js_version: env_string_default("JS_VERSION", "1"),
            assets_domain: env_string_optional("ASSETS_DOMAIN"),
        }
    }
}

pub fn dev_mode<'a>() -> bool {
    CONFIG.dev_mode
}

pub fn database_url<'a>() -> &'a str {
    &CONFIG.database_url
}

pub fn git_sha<'a>() -> &'a str {
    &CONFIG.git_sha
}

pub fn sentry_dsn<'a>() -> &'a Option<String> {
    &CONFIG.sentry_dsn
}
pub fn static_dir<'a>() -> &'a Path {
    &CONFIG.static_dir
}

pub fn blogs_dir<'a>() -> &'a Path {
    &CONFIG.blogs_dir
}

pub fn cms_dir<'a>() -> &'a Path {
    &CONFIG.cms_dir
}
pub fn search_index_dir<'a>() -> &'a Path {
    &CONFIG.search_index_dir
}
pub fn render_errors<'a>() -> bool {
    CONFIG.render_errors
}

pub fn deployment<'a>() -> &'a str {
    &CONFIG.deployment
}
pub fn signup_url<'a>() -> &'a str {
    &CONFIG.signup_url
}
pub fn standalone_dashboard<'a>() -> bool {
    CONFIG.standalone_dashboard
}

pub fn github_stars<'a>() -> &'a str {
    &CONFIG.github_stars
}

pub fn css_url() -> String {
    if CONFIG.dev_mode {
        return "/dashboard/static/css/style.css".to_string();
    }

    let path = format!("/dashboard/static/css/style.{}.css", CONFIG.css_version);
    asset_url(&path)
}

pub fn js_url(name: &str) -> String {
    if CONFIG.dev_mode {
        return format!("/dashboard/static/js/{name}");
    }

    let name = name.split(".").collect::<Vec<&str>>();
    let name = name[0..name.len() - 1].join(".");

    let path = format!("/dashboard/static/js/{name}.{}.js", CONFIG.js_version);

    asset_url(&path)
}

fn asset_url(path: &str) -> String {
    match &CONFIG.assets_domain {
        Some(domain) => format!("https://{domain}{path}"),
        None => path.to_string(),
    }
}

fn env_is_set(name: &str) -> bool {
    match var(name) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn env_string_required(name: &str) -> String {
    var(name)
        .expect(&format!(
            "{} env variable is required for proper configration",
            name
        ))
        .to_string()
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
