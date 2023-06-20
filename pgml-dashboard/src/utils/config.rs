use std::env::var;

pub fn dev_mode() -> bool {
    match var("DEV_MODE") {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn database_url() -> String {
    match var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => "postgres:///pgml".to_string(),
    }
}

pub fn git_sha() -> String {
    env!("GIT_SHA").to_string()
}

pub fn sentry_dsn() -> Option<String> {
    match var("SENTRY_DSN") {
        Ok(dsn) => Some(dsn),
        Err(_) => None,
    }
}

pub fn static_dir() -> String {
    match var("DASHBOARD_STATIC_DIRECTORY") {
        Ok(dir) => dir,
        Err(_) => "static".to_string(),
    }
}

pub fn content_dir() -> String {
    match var("DASHBOARD_CONTENT_DIRECTORY") {
        Ok(dir) => dir,
        Err(_) => "content".to_string(),
    }
}

pub fn search_index_dir() -> String {
    match var("SEARCH_INDEX_DIRECTORY") {
        Ok(path) => path,
        Err(_) => "search_index".to_string(),
    }
}

pub fn render_errors() -> bool {
    match var("RENDER_ERRORS") {
        Ok(_) => true,
        Err(_) => dev_mode(),
    }
}

pub fn deployment() -> String {
    match var("DEPLOYMENT") {
        Ok(env) => env,
        Err(_) => "localhost".to_string(),
    }
}

pub fn css_url() -> String {
    if dev_mode() {
        return "/dashboard/static/css/style.css".to_string();
    }

    let filename = format!("style.{}.css", env!("CSS_VERSION"));

    let path = format!("/dashboard/static/css/{filename}");

    match var("ASSETS_DOMAIN") {
        Ok(domain) => format!("https://{domain}{path}"),
        Err(_) => path,
    }
}

pub fn js_url(name: &str) -> String {
    let name = if dev_mode() {
        name.to_string()
    } else {
        let name = name.split(".").collect::<Vec<&str>>();
        let name = name[0..name.len() - 1].join(".");
        format!("{name}.{}.js", env!("JS_VERSION"))
    };

    let path = format!("/dashboard/static/js/{name}");

    match var("ASSETS_DOMAIN") {
        Ok(domain) => format!("https://{domain}{path}"),
        Err(_) => path,
    }
}

pub fn signup_url() -> String {
    if dev_mode() {
        "/signup".to_string()
    } else {
        "https://postgresml.org/signup".to_string()
    }
}

pub fn standalone_dashboard() -> bool {
    !env!("CARGO_MANIFEST_DIR").contains("deps") && !env!("CARGO_MANIFEST_DIR").contains("cloud2")
}
