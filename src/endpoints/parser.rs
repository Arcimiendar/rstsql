use std::{fs::read_dir, path::PathBuf};

use log::warn;
use rstmytype::{ApiEndpoint, ApiEndpointMethod, ApiProject};

#[derive(Debug)]
pub struct Endpoint {
    pub tag: String,
    pub method: ApiEndpointMethod,
    pub url_path: String,
    pub file_content: String,
    pub schema: String,
}

impl Endpoint {
    fn new(
        tag: String,
        method: ApiEndpointMethod,
        file_path: &PathBuf,
        relative_url_path: &str,
    ) -> Option<Endpoint> {
        let file_content = std::fs::read_to_string(file_path).ok()?;

        let schema = if Endpoint::contains_schema(&file_content) {
            Endpoint::extract_schema(&file_content)
        } else {
            "".to_string()
        };

        let url_path = format!("{}/{}", relative_url_path, file_path.file_stem()?.to_str()?);

        Some(Endpoint {
            tag,
            method,
            url_path,
            file_content,
            schema,
        })
    }

    fn parse_from_dir_rec(
        method: &ApiEndpointMethod,
        tag: &str,
        current_dir: &PathBuf,
        current_url: &str,
        endpoints_acc: &mut Vec<Endpoint>,
    ) {
        read_dir(current_dir)
            .ok()
            .iter_mut()
            .flat_map(|r| r.into_iter())
            .flat_map(|r| r.ok())
            .for_each(|f| {
                let path = f.path();
                if path.is_dir() {
                    let Some(url) = f
                        .file_name()
                        .to_str()
                        .map(|f_name| format!("{}/{}", current_url, f_name))
                    else {
                        return;
                    };

                    Self::parse_from_dir_rec(method, tag, &path, &url, endpoints_acc);
                    return;
                }

                if path.is_file() {
                    let endp = Endpoint::new(tag.to_string(), method.clone(), &path, current_url);

                    if let Some(endpoint) = endp {
                        endpoints_acc.push(endpoint);
                    }
                    return;
                }

                warn!("{} is not a sql file or dir", path.display());
            });
    }

    pub fn contains_schema(file_content: &str) -> bool {
        file_content.starts_with("/*")
    }

    pub fn extract_schema(file_content: &str) -> String {
        let mut result = String::with_capacity(file_content.len());
        let mut chars = file_content.chars().peekable();
        // skip initial "/*"
        chars.next();
        chars.next();

        while let Some(c) = chars.next() {
            if c == '*' {
                // Peek to check if this is an end of declaration
                if chars.peek() == Some(&'/') {
                    break;
                }
            }
            result.push(c);
        }

        result.clone() // clone to dealocate extra capacity
    }
}

#[derive(Debug)]
pub struct EndpointCollections {
    pub endpoints: Vec<Endpoint>,
}

impl EndpointCollections {
    pub fn parse_from_dir(dsl_dir: &str) -> Self {
        let mut endpoints_acc = Vec::new();
        let current_url = "";
        let current_dir = PathBuf::from(dsl_dir);
        Self::parse_from_dir_rec(&current_dir, current_url, &mut endpoints_acc);
        Self {
            endpoints: endpoints_acc,
        }
    }

    fn parse_from_dir_rec(
        current_dir: &PathBuf,
        current_url: &str,
        endpoints_acc: &mut Vec<Endpoint>,
    ) {
        read_dir(current_dir)
            .ok()
            .iter_mut()
            .flat_map(|r| r.into_iter())
            .flat_map(|e| e.ok())
            .for_each(|e| {
                let path = e.path();
                if !path.is_dir() {
                    warn!(
                        "Skipping file {} because it's not a dir",
                        e.path().display()
                    );
                    return;
                }

                if e.file_name() == "GET" {
                    Endpoint::parse_from_dir_rec(
                        &ApiEndpointMethod::Get,
                        current_url,
                        &path,
                        current_url,
                        endpoints_acc,
                    );
                    return;
                }

                if e.file_name() == "POST" {
                    Endpoint::parse_from_dir_rec(
                        &ApiEndpointMethod::Post,
                        current_url,
                        &path,
                        current_url,
                        endpoints_acc,
                    );
                    return;
                }

                let Some(url) = e
                    .file_name()
                    .to_str()
                    .map(|f_name| format!("{}/{}", current_url, f_name))
                else {
                    return;
                };

                Self::parse_from_dir_rec(&path, &url, endpoints_acc);
            });
    }
}

impl std::fmt::Display for EndpointCollections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let endpoints_strings: Vec<String> =
            self.endpoints.iter().map(|e| format!("{}", e)).collect();
        write!(f, "{{ endpoints: [{}] }}", endpoints_strings.join(","))
    }
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ method: {:?}, url: {} }}", self.method, self.url_path)
    }
}

impl ApiEndpoint for Endpoint {
    fn get_url_path(&self) -> &str {
        &self.url_path
    }

    fn get_endpoint_method(&self) -> &ApiEndpointMethod {
        &self.method
    }

    fn get_yml_declaration_str(&self) -> Option<&str> {
        if self.schema.is_empty() {
            return None;
        }

        Some(&self.schema)
    }

    fn get_endpoint_tag(&self) -> &str {
        &self.tag
    }
}

impl ApiProject for EndpointCollections {
    fn get_title(&self) -> &str {
        "rstsql"
    }

    fn get_endpoints_iter(&self) -> impl Iterator<Item = &impl ApiEndpoint> {
        self.endpoints.iter()
    }
}
