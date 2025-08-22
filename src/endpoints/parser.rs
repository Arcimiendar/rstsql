use std::fs::DirEntry;

use rstmytype::{ApiProject, ApiEndpointMethod, ApiEndpoint};
use log::warn;

#[derive(Debug, Clone, PartialEq)]
pub enum EndpointMethod {
    GET,
    POST,
}

#[derive(Debug)]
pub struct Endpoint {
    pub tag: String,
    pub method: EndpointMethod,
    pub url_path: String,
    pub file_content: String,
    pub schema: String
}

impl Endpoint {
    fn new(
        tag: String,
        method: EndpointMethod,
        file_path: &String,
        url_path: String,
    ) -> Option<Endpoint> {
        let content = std::fs::read_to_string(&file_path).ok()?;

        let schema = if Endpoint::contains_schema(&content) {
            Endpoint::extract_schema(&content)
        } else {
            "".to_string()
        };

        Some(Endpoint {
            tag,
            method,
            url_path,
            file_content: content,
            schema,
        })
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
pub struct Project {
    pub project_name: String,
    pub endpoints: Vec<Endpoint>,
}

impl Project {
    fn load_enpoints(
        tag: &str,
        rel_path: String,
        e: &DirEntry,
        method: &EndpointMethod,
    ) -> Option<Box<dyn Iterator<Item = Endpoint>>> {
        let paths = std::fs::read_dir(e.path()).ok()?;
        let iter = paths
            .into_iter()
            .flat_map(|e| e.ok())
            .filter(|e| {
                e.path().is_dir()
                    || match e.path().to_str() {
                        Some(s) => s.ends_with(".sql"),
                        None => false,
                    }
            })
            .map(|e| {
                if e.path().is_dir() {
                    return Project::load_enpoints(
                        &tag,
                        format!("{}/{}", rel_path, e.file_name().to_str()?),
                        &e,
                        method,
                    );
                } else {
                    let filename = e.file_name().into_string().ok()?;
                    let len = filename.len();
                    let endpoint = Endpoint::new(
                        tag.to_string(),
                        method.clone(),
                        &e.path().to_str()?.to_string(),
                        format!("/{}/{}", rel_path, filename[..len - 4].to_string()),
                    )?;
                    return Some(Box::new(Some(endpoint).into_iter()));
                }
            })
            .flat_map(|r| r)
            .reduce(|a, b| Box::new(a.chain(b)))
            .or(Some(Box::new(std::iter::empty())));

        iter
    }

    fn load_get_enpoints(
        rel_path: &String,
        e: &DirEntry,
    ) -> Option<Box<dyn Iterator<Item = Endpoint>>> {
        Project::load_enpoints(&rel_path, rel_path.clone(), e, &EndpointMethod::GET)
    }

    fn load_post_endpoints(
        rel_path: &String,
        e: &DirEntry,
    ) -> Option<Box<dyn Iterator<Item = Endpoint>>> {
        Project::load_enpoints(&rel_path, rel_path.clone(), e, &EndpointMethod::POST)
    }

    pub fn parse_from_dir_entry(entry: &DirEntry) -> Option<Project> {
        let name = entry.file_name().to_str()?.to_string();
        let paths = std::fs::read_dir(entry.path()).ok()?;

        let iter = paths
            .flat_map(|e| e.ok())
            .filter(|e| {
                if e.path().is_file() {
                    warn!(
                        "Skipping project {} has unexpected file {}",
                        name,
                        e.path().display()
                    );
                    return false;
                }

                if let Some(file_str) = e.file_name().to_str() {
                    if ["GET", "POST"].contains(&file_str) {
                        true
                    } else {
                        warn!("Skipping project {} unsupported method {}", name, file_str);

                        false
                    }
                } else {
                    false
                }
            })
            .map(|e| {
                if e.path().ends_with("GET") {
                    Project::load_get_enpoints(&name, &e)
                } else if e.path().ends_with("POST") {
                    Project::load_post_endpoints(&name, &e)
                } else {
                    panic!("something went wrong in parse_from_dir_entry")
                }
            })
            .flat_map(|r| r)
            .reduce(|a, b| Box::new(a.chain(b)))?;

        Some(Project {
            project_name: name,
            endpoints: iter.collect(),
        })
    }
}

#[derive(Debug)]
pub struct EndpointCollections {
    pub projects: Vec<Project>,
}

impl EndpointCollections {
    pub fn parse_from_dir(dsl_dir: &String) -> EndpointCollections {
        let paths = std::fs::read_dir(&dsl_dir);

        let projects = paths
            .ok()
            .iter_mut()
            .flat_map(|r| r.into_iter())
            .flat_map(|e| e.ok())
            .filter(|e| {
                let valid = e.path().is_dir();
                if !valid {
                    warn!(
                        "Skipping file {} because it's not a project",
                        e.path().display()
                    );
                }
                valid
            })
            .map(|e| Project::parse_from_dir_entry(&e))
            .flat_map(|r| r)
            .collect();

        EndpointCollections { projects: projects }
    }
}

impl std::fmt::Display for EndpointCollections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let projects_strings: Vec<String> =
            self.projects.iter().map(|p| format!("{}", p)).collect();

        write!(f, "{{ projects: [{}] }}", projects_strings.join(","))
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let endpoints_strings: Vec<String> =
            self.endpoints.iter().map(|e| format!("{}", e)).collect();
        write!(
            f,
            "{{ name: {}, endpoints: [{}] }}",
            self.project_name,
            endpoints_strings.join(",")
        )
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
        match self.method {
            EndpointMethod::GET => &ApiEndpointMethod::Get,
            EndpointMethod::POST => &ApiEndpointMethod::Post
        }
    }

    fn get_yml_declaration_str(&self) -> Option<&str> {
        if self.schema.len() == 0 {
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

    fn get_endpoints_iter<'a>(&'a self) -> impl Iterator<Item = &'a impl ApiEndpoint> {
        self.projects.iter()
        .flat_map(|p| p.endpoints.iter())
        
    }
}