use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

lazy_static! {
    static ref UUID_REGEX: Regex =
        Regex::new("[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}").unwrap();
}

pub struct ClusterUrlParser;

impl ClusterUrlParser {
    // Returns option containing path. If regex does not match, returns None.
    pub fn get_path(url: &str) -> Option<String> {
        let url = Url::parse(&url);
        let mut path = String::new();

        match url {
            Ok(url) => {
                path.push_str(url.path());

                match url.query() {
                    Some(query) => {
                        path.push_str("?");
                        path.push_str(query)
                    }
                    None => (),
                }

                match url.fragment() {
                    Some(fragment) => {
                        path.push_str("#");
                        path.push_str(fragment)
                    }
                    None => (),
                }

                Some(path)
            }
            Err(_) => None,
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
    fn parse_local_url_2() {
        let url = "http://127.0.0.1/clusters/database1/notebooks";
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
    fn parse_non_postgres_url() {
        let url = "https://example.com/clusters/database1/notebooks";
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
        let url = "qeywroqyfrebptdgqiuyhracpvbfij";
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
    fn parse_with_fragment() {
        let url = "https://prefix.postgresml.org/clusters/database1/notebooks?tag=notebook#row=4";
        assert_eq!(
            ClusterUrlParser::get_path(url),
            Some("/clusters/database1/notebooks?tag=notebook#row=4".to_string())
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
