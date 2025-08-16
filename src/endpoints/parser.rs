use std::fs::DirEntry;

use log::warn;


#[derive(Debug, Clone, PartialEq)]
pub enum EndpointMethod { GET, POST }




#[derive(Debug)]
pub struct Endpoint {
    pub method: EndpointMethod,
    pub url_path: String,
    pub file_content: String,

}

impl Endpoint {

    fn new(method: EndpointMethod, file_path: &String, url_path: String) -> Endpoint {

        let content = std::fs::read_to_string(&file_path).unwrap();

        Endpoint { 
            method: method, 
            url_path: url_path,
            file_content: content,
        }
    }

}

#[derive(Debug)]
pub struct Project {
    pub project_name: String,
    pub endpoints: Vec<Endpoint>,
}

impl Project {
    fn load_enpoints(rel_path: String, e: &DirEntry, method: &EndpointMethod) -> Box<dyn Iterator<Item = Endpoint>> {
        let paths = std::fs::read_dir(e.path()).unwrap();
        let iter = paths
            .into_iter()
            .map(|e| e.unwrap())
            .filter(|e| e.path().is_dir() || e.path().to_str().unwrap().ends_with(".sql"))
            .map(|e| {
                if e.path().is_dir() {

                    return Project::load_enpoints( 
                        format!(
                            "{}/{}", rel_path, e.file_name().to_str().unwrap()
                        ), &e, method
                    );
                } else {
                    let filename = e.file_name().into_string().unwrap();
                    let len = filename.len();
                    let endpoint = Endpoint::new(
                        method.clone(), 
                        &e.path().to_str().unwrap().to_string(),
                        format!("/{}/{}", rel_path, filename[..len-4].to_string())
                    );
                    return Box::new(Some(endpoint).into_iter());
                }
            })
            .reduce(|a, b| Box::new(a.chain(b)))
            .or(Some(Box::new(std::iter::empty()))).unwrap();

        iter
    }


    fn load_get_enpoints(rel_path: &String, e: &DirEntry) -> Box<dyn Iterator<Item = Endpoint>> {
        Project::load_enpoints(rel_path.clone(), e, &EndpointMethod::GET)
    }

    fn load_post_endpoints(rel_path: &String, e: &DirEntry) -> Box<dyn Iterator<Item = Endpoint>> {
        Project::load_enpoints(rel_path.clone(), e, &EndpointMethod::POST)
    }

    pub fn parse_from_dir_entry(entry: &DirEntry) -> Project {
        let name = entry.file_name().to_str().unwrap().to_string();
        let paths = std::fs::read_dir(entry.path()).unwrap();
 
        let iter = paths
            .map(|e| e.unwrap())
            .filter(|e| {
                if e.path().is_file() {
                    warn!("Skipping project {} has unexpected file {}", name, e.path().display());
                    false
                } else if ["GET", "POST"].contains(&e.file_name().to_str().unwrap()) {
                    true
                } else {
                    warn!("Skipping project {} unsupported method {}", name, e.file_name().to_str().unwrap());
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
            .reduce(|a, b| Box::new(a.chain(b))).unwrap();
        
        Project {
            project_name: name,
            endpoints: iter.collect(),
        }

        
    }
}

#[derive(Debug)]
pub struct EndpointCollections {
    pub projects: Vec<Project>,
}

impl EndpointCollections {
    pub fn parse_from_dir(dsl_dir: &String) -> EndpointCollections {
        let paths = std::fs::read_dir(&dsl_dir).unwrap();

        let projects = paths.into_iter()
            .map(|e| e.unwrap())
            .filter(|e| {
                let valid = e.path().is_dir();
                if !valid {
                    warn!("Skipping file {} because it's not a project", e.path().display());
                }
                valid
            })
            .map(
                |e| Project::parse_from_dir_entry(&e)
            )
            .collect();

        EndpointCollections { projects: projects }
    }
}

impl std::fmt::Display for EndpointCollections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let projects_strings: Vec<String> = self.projects.iter().map(|p| format!("{}", p)).collect();

        write!(f, "{{ projects: [{}] }}", projects_strings.join(","))
    }
}


impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let endpoints_strings: Vec<String> = self.endpoints.iter().map(|e| format!("{}", e)).collect();
        write!(f, "{{ name: {}, endpoints: [{}] }}", self.project_name, endpoints_strings.join(","))
    }
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ method: {:?}, url: {} }}", self.method, self.url_path)
    }
}