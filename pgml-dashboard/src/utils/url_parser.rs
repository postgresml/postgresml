use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref DOMAIN_REGEX: Regex =
        Regex::new(r"^https?://((localhost:8000)|(.*\.?postgresml\.org))").unwrap();
    static ref UUID_REGEX: Regex = Regex::new(".{8}-.{4}-.{4}-.{4}-.{12}").unwrap();
}

pub struct ClusterUrlParser;

impl ClusterUrlParser {
    // Returns option containing path. If regex does not match, returns None.
    pub fn get_path(url: &str) -> Option<String> {
        let path = DOMAIN_REGEX.replace(url.clone(), "").to_string();
        match url.eq(&path) {
            true => None,
            _ => Some(path),
        }
    }

    // Finds the uuid in a url.
    pub fn get_uuid(uri: String) -> Option<String> {
        let uuid = UUID_REGEX.find(&uri);
        match uuid {
            Some(uuid) => Some(uri.to_string()[uuid.start()..uuid.end()].to_string()),
            None => None,
        }
    }

    // Determines if there is a uuid in a url.
    pub fn has_uuid(uri: String) -> bool {
        match UUID_REGEX.find(&uri) {
            Some(_) => true,
            _ => false,
        }
    }

    // Replaces the first uuid with a new uuid in a url.
    pub fn replace_path_uuid(path: String, uuid: String) -> String {
        UUID_REGEX.replace(&path, uuid).to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::utils::url_parser::ClusterUrlParser;

    #[test]
    fn parse_local_url() {
        let url = "http://localhost:8000/clusters/database1/notebooks";
        assert_eq!(
            ClusterUrlParser::get_path(url),
            Some("/clusters/database1/notebooks".to_string())
        )
    }

    #[test]
    fn parse_postgres_url() {
        let url = "https://postgresml.org/clusters/database1/notebooks";
        assert_eq!(
            ClusterUrlParser::get_path(url),
            Some("/clusters/database1/notebooks".to_string())
        )
    }

    #[test]
    fn parse_prefix_url() {
        let url = "https://prefix.postgresml.org/clusters/database1/notebooks";
        assert_eq!(
            ClusterUrlParser::get_path(url),
            Some("/clusters/database1/notebooks".to_string())
        )
    }

    #[test]
    fn parse_subdomain_url() {
        let url = "https://www.prefix.postgresml.org/clusters/database1/notebooks";
        assert_eq!(
            ClusterUrlParser::get_path(url),
            Some("/clusters/database1/notebooks".to_string())
        )
    }

    #[test]
    fn bad_url() {
        let url = "https://test.org/clusters/database1/notebooks";
        assert_eq!(ClusterUrlParser::get_path(url), None)
    }

    #[test]
    fn parse_with_params() {
        let url = "https://prefix.postgresml.org/clusters/database1/notebooks?tag=notebook";
        assert_eq!(
            ClusterUrlParser::get_path(url),
            Some("/clusters/database1/notebooks?tag=notebook".to_string())
        )
    }

    #[test]
    fn has_uuid_false() {
        let url = "https://prefix.postgresml.org/clusters/database1/notebooks?tag=notebook";
        assert!(!ClusterUrlParser::has_uuid(url.to_string()))
    }

    #[test]
    fn has_uuid_true() {
        let url = "https://prefix.postgresml.org/clusters/430ae78c-271c-4493-aa4e-deda881a072f/notebooks?tag=notebook";
        assert!(ClusterUrlParser::has_uuid(url.to_string()))
    }

    #[test]
    fn replace_uuid() {
        let url =
            "/clusters/430ae78c-271c-4493-aa4e-deda881a072f/notebooks?tag=notebook".to_string();
        let uuid = "1234567a-bcde-1234-abcd-1a2b3c4d5e6f".to_string();
        let ault_url = format!("/clusters/{}/notebooks?tag=notebook", uuid);
        assert_eq!(ClusterUrlParser::replace_path_uuid(url, uuid), ault_url)
    }

    #[test]
    fn replace_uuid_no_uuid() {
        let url = "/clusters/notebooks?tag=notebook".to_string();
        let uuid = "1234567a-bcde-1234-abcd-1a2b3c4d5e6f".to_string();
        let ault_url = format!("/clusters/notebooks?tag=notebook");
        assert_eq!(ClusterUrlParser::replace_path_uuid(url, uuid), ault_url)
    }
}
